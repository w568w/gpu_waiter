#![feature(try_blocks)]
use std::{
    ffi::OsString,
    num::NonZeroU32,
    process::Command,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use clap::{Parser, Subcommand};
use crossbeam_channel::{select, never};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::{error, info, warn};
use mimalloc::MiMalloc;
use nvml_wrapper::Nvml;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;

mod lock;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[command(version, about, author, long_about = None)]
#[command(
    help_template = "{before-help}{about-with-newline}\nAuthor: {author-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
/// A simple tool to wait for idle GPUs, occupy them, and run a given command.
struct Cli {
    /// How many GPUs to use
    #[arg(short, long, default_value_t=NonZeroU32::new(1).unwrap())]
    num: NonZeroU32,

    /// An external command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

static NVML: OnceCell<Nvml> = OnceCell::new();

fn get_idle_gpu() -> anyhow::Result<Vec<u32>> {
    let nvml = NVML.wait();
    let device_count = nvml.device_count()?;
    let mut result = Vec::with_capacity(device_count as usize);
    for i in 0..device_count {
        let device = nvml.device_by_index(i)?;
        if device.running_compute_processes_count()? == 0 {
            result.push(i);
        }
    }
    Ok(result)
}

static STOPPED: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    let logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();
    LogWrapper::new(multi.clone(), logger).try_init()?;

    if let Err(err) = ctrlc::set_handler(move || {
        info!("Ctrl+C received, exiting...");
        STOPPED.store(true, std::sync::atomic::Ordering::Relaxed);
    }) {
        warn!("Failed to set Ctrl+C handler: {}", err)
    }

    if std::env::var("CUDA_VISIBLE_DEVICES").is_ok() {
        warn!("CUDA_VISIBLE_DEVICES is already set, which will be ignored");
        std::env::remove_var("CUDA_VISIBLE_DEVICES");
    }
    NVML.get_or_try_init(Nvml::init)?;

    let args = Cli::parse();
    let nvml = Nvml::init()?;
    let device_count = nvml.device_count()?;
    if args.num.get() > device_count {
        return Err(anyhow::anyhow!(
            "Requested {} devices, but there are only {} devices in total",
            args.num,
            device_count
        ));
    }

    // start waiting
    info!(
        "Start waiting at {}",
        chrono::Local::now().format("%H:%M:%S")
    );
    // show a spinner for polling
    let spinner = multi.add(indicatif::ProgressBar::new_spinner());
    spinner.set_message("Waiting for idle GPUs...");
    spinner.enable_steady_tick(Duration::from_millis(500));
    let mut idle_gpu = None;
    // init global file lock
    let file_lock = lock::FileRWLock::new("gpu-waiter.lock")?;
    let mut lock_guard = None;
    // poll for idle GPUs
    while !STOPPED.load(std::sync::atomic::Ordering::Relaxed) {
        let guard_in_loop = file_lock.write()?;
        let mut idle_gpus = get_idle_gpu()?;
        if idle_gpus.len() >= args.num.get() as usize {
            info!("Found {} idle GPUs!: {:?}", args.num, idle_gpus);
            idle_gpus.splice(args.num.get() as usize.., std::iter::empty());
            idle_gpu = Some(idle_gpus);
            lock_guard = Some(guard_in_loop);
            break;
        }
        drop(guard_in_loop);
        spinner.set_message(format!(
            "Waiting for idle GPUs... ({} available, {} requested) [Last check: {}]",
            idle_gpus.len(),
            args.num,
            chrono::Local::now().format("%H:%M:%S")
        ));
        thread::sleep(Duration::from_secs(1));
    }

    // remove the spinner
    spinner.finish_and_clear();
    multi.remove(&spinner);

    if let Some(idle_gpu) = idle_gpu {
        info!("Occupying GPUs: {:?}", idle_gpu);

        let (device_used_s, device_used_r) = crossbeam_channel::unbounded();
        let (proc_exit_s, proc_exit_r) = crossbeam_channel::unbounded();
        let occupantions = Arc::new(RwLock::new(Vec::with_capacity(idle_gpu.len())));
        for i in &idle_gpu {
            let cuda_dev = cudarc::driver::CudaDevice::new(*i as usize)?;
            let nvml_dev = NVML.wait().device_by_index(*i)?;
            let free_mem = nvml_dev.memory_info()?.free;

            let out = cuda_dev.alloc_zeros::<u8>((free_mem / 4) as usize)?;
            occupantions.write().push((*i, out));
        }

        info!("GPUs occupied: {:?}", idle_gpu);
        // after occupying, drop the lock guard
        if let Some(guard) = lock_guard {
            drop(guard);
        }

        let occp = occupantions.clone();
        thread::spawn(move || {
            'outer: while occp.read().len() > 0 {
                for (i, _) in occp.read().iter() {
                    let result: anyhow::Result<()> = try {
                        let nvml_dev = NVML.wait().device_by_index(*i)?;
                        if nvml_dev.running_compute_processes_count()? > 1 {
                            if let Err(e) = device_used_s.send(Ok(*i)) {
                                error!("Failed to send used device: {}", e);
                                break 'outer;
                            }
                        }
                    };

                    if let Err(err) = result {
                        let _ = device_used_s.send(Err(err));
                        break 'outer;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });

        let Commands::External(cmds) = args.command;
        let mut cmd = Command::new(&cmds[0])
            .args(&cmds[1..])
            .env(
                "CUDA_VISIBLE_DEVICES",
                idle_gpu
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .spawn()?;
        thread::spawn(move || {
            let _ = proc_exit_s.send(cmd.wait());
        });
        
        let mut device_used_r = Some(&device_used_r);
        'select: while !STOPPED.load(std::sync::atomic::Ordering::Relaxed) {
            select! {
                recv(device_used_r.unwrap_or(&never())) -> res => {
                    if matches!(res, Err(_)) {
                        device_used_r = None;
                    } else {
                        let used_index = res??;
                        occupantions.write().retain(|(j, _)| *j != used_index);
                    }
                }
                recv(proc_exit_r) -> res => {
                    let status = res??;
                    info!("Process exited with status: {}", status);
                    break 'select;
                }
            }
        }
    }
    Ok(())
}
