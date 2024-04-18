# GPU Waiter

Automatically wait for a NVIDIA GPU (or some) to be available, and then run a program. No more manual queueing with executing `nvidia-smi` again and again!

- Written in Rust; robust and fast
- "Available" means the GPU is not being used by any other computing program
- Only depends on CUDA and NVML libraries
- Support waiting for multiple GPUs
- Automatically hold the GPUs for you - by occupying them with 1/4 free GPU memory; automatically release them when the target program starts using GPUs

## How would you compare it with X?
There are some similar tools like [NvidiaGPUWaiter](https://github.com/andsfonseca/NvidiaGPUWaiter) and [GrabGPU](https://github.com/godweiyang/GrabGPU/). The main differences are:

- NvidiaGPUWaiter is a python script and can be only used in python file. It does not provide a command line interface.
- GrabGPU is a cli tool written in CUDA CXX. It does not check any errors, which is a bad practice when interacting with CUDA API. It also includes unnecessary workloads to occupy the GPU, and does not consider concurrency issues at all.

## Usage

Just build and run:

```bash
# Wait for one GPU and run "python my_program.py"
$ gpu-waiter python my_program.py
# Wait for two GPUs and run "env"
$ gpu-waiter -n 2 env
# Wait for one GPU and run "env" with environment variables
$ SOME_VAR=1 gpu-waiter env
```

## Caveats

- Not all concurrency scenarios are tested. There could be chances that A and B grab the same GPU at the same time. Planning to avoid this with a lock mechanism.

# GPU 排队器

自动等待一个（或一些）NVIDIA GPU 可用，然后运行一个程序。不再需要手动查看 `nvidia-smi` 排队！

- 用 Rust 编写；稳定快速
- "可用" 意味着 GPU 没有被其他任何计算程序使用
- 仅依赖 CUDA 和 NVML 库
- 支持等待多个 GPU
- 自动为你占用 GPU - 用 1/4 空闲显存占住 GPU；当目标程序开始使用 GPU 时，自动释放被占用的显存

## 与其他工具的比较
有一些类似的工具，如 [NvidiaGPUWaiter](https://github.com/andsfonseca/NvidiaGPUWaiter) 和 [GrabGPU](https://github.com/godweiyang/GrabGPU/)。主要区别在于：

- NvidiaGPUWaiter 是一个 Python 脚本，只能在 Python 文件中使用。它不提供命令行工具。
- GrabGPU 是一个用 CUDA CXX 编写的命令行工具。它没有检查任何运行时错误，假定一切正常运行，这在与 CUDA 交互时是一个不好的实践。它还包含了不必要的工作负载来占用 GPU，并且完全不考虑并发问题。

## 用法

编译并运行：

```bash
# 等待一个 GPU 并运行 "python my_program.py"
$ gpu-waiter python my_program.py
# 等待两个 GPU 并运行 "env"
$ gpu-waiter -n 2 env
# 等待一个 GPU 并运行带环境变量的 "env"
$ SOME_VAR=1 gpu-waiter env
```

## 缺陷

- 并非所有并发场景都经过测试，A 和 B 同时抢占同一个 GPU 也是有可能的。计划通过锁机制避免这种情况。