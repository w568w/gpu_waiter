# GPU Waiter

Automatically wait for a NVIDIA GPU (or some) to be available, and then run a program. No more manual queueing with executing `nvidia-smi` again and again!

- Written in Rust; robust and fast
- Only depends on CUDA and NVML libraries
- Support waiting for multiple GPUs
- Automatically hold the GPUs for you - by occupying them with 1/4 free GPU memory; automatically release them when the target program starts using GPUs

# 等 GPU

自动等待一个（或一些）NVIDIA GPU 可用，然后运行一个程序。不再需要手动查看 `nvidia-smi` 排队！

- 用 Rust 编写；稳定快速
- 仅依赖 CUDA 和 NVML 库
- 支持等待多个 GPU
- 自动为你占用 GPU - 用 1/4 空闲显存占住 GPU；当目标程序开始使用 GPU 时，自动释放被占用的显存