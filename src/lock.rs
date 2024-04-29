use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use fs4::FileExt;
use log::warn;

/// A heuristic way to decide a global runtime directory.
fn guess_global_runtime_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("C:\\ProgramData")
    } else if cfg!(target_os = "android") {
        PathBuf::from("/data/local/tmp")
    } else if cfg!(unix) {
        PathBuf::from("/tmp")
    } else {
        panic!("You are running on an unsupported platform!")
    }
}

pub struct FileRWLock {
    file: std::fs::File,
}

pub struct RWLockReadGuard<'a> {
    _lock: &'a FileRWLock,
}

pub struct RWLockWriteGuard<'a> {
    _lock: &'a FileRWLock,
}

/// Tries to open a file, if it does not exist, create it.
///
/// # Special case: Why not just use [`File::create`], which does the same thing?
/// Linux's `fs.protected_regular != 0` will prevent `create` to open an existing file (i.e. `O_CREAT`)
/// in a other/group-writable sticky directory, unless the file is owned by the directory's owner.
///
/// Unfortunately, this is the case for `/tmp` on many Linux distributions. So we have to try [`File::open`]
/// first and then `create` if it fails with `NotFound`.
fn open_or_create_file(path: impl AsRef<Path>) -> io::Result<File> {
    let p = path.as_ref();

    // 1. try to open the file
    let f = File::open(p);

    match f {
        // 2.1. if we can open the file, return it
        Ok(f) => Ok(f),
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                // 2.2. if we can't open the file because of other reasons, return the error
                Err(e)
            } else {
                // 2.3. the file does not exist, try to create the file atomically
                match File::create_new(p) {
                    Ok(f) => {
                        // 3.1 if we can create the file, make it world-writable (unnecessary?) on Unix
                        if cfg!(unix) {
                            let r = f.set_permissions(<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o777));
                            if let Err(e) = r {
                                warn!("Failed to set permissions of file {:?}: {:?}", p, e);
                            }
                        }
                        Ok(f)
                    },
                    Err(e) => {
                        if e.kind() == io::ErrorKind::AlreadyExists {
                            // 3.2. the file has been created by another process, try to open it
                            File::open(p)
                        } else {
                            // 3.3. other creation errors
                            Err(e)
                        }
                    }
                }
            }
        }
    }
}

impl FileRWLock {
    pub fn new(name: impl AsRef<Path>) -> io::Result<Self> {
        let base_rt_dir = guess_global_runtime_dir();
        if !base_rt_dir.exists() {
            panic!(
                "The guessed runtime directory on your platform does not exist: {:?}",
                base_rt_dir
            );
        }
        let p = base_rt_dir.join(name);
        let f = open_or_create_file(&p)?;
        Ok(Self { file: f })
    }

    pub fn read(&self) -> io::Result<RWLockReadGuard<'_>> {
        self.file.lock_shared()?;
        Ok(RWLockReadGuard { _lock: self })
    }

    pub fn write(&self) -> io::Result<RWLockWriteGuard<'_>> {
        self.file.lock_exclusive()?;
        Ok(RWLockWriteGuard { _lock: self })
    }
}

impl Drop for RWLockReadGuard<'_> {
    fn drop(&mut self) {
        self._lock.file.unlock().expect("Failed to unlock file");
    }
}

impl Drop for RWLockWriteGuard<'_> {
    fn drop(&mut self) {
        self._lock.file.unlock().expect("Failed to unlock file");
    }
}
