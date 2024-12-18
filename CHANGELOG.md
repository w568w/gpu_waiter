# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-11-22
### Added
- [BREAKING] Add a new feature to allow the user to dynamically insert GPU IDs to their command line arguments. A new argument `-f` to force `CUDA_VISIBLE_DEVICES` to be set while inserting GPU IDs at the same time.

### Fixed
- The external command exits immediately after it starts using the GPU, because:
    - We only waited for the first message, with a `select!` without loop;
    - We didn't catch the disconnected error after the GPU monitor thread exits.
- Really "ignore" `CUDA_VISIBLE_DEVICES` environment variable by explicitly removing it.
- On Linux distributions with `fs.protected_regular` enabled, we will fail to `create` (i.e. open) an existing lock file.
- On Linux the NVML library path is being migrated to `libnvidia-ml.so.1` from `libnvidia-ml.so`. See <https://github.com/Cldfire/nvml-wrapper/pull/63> for details.

## [0.1.1] - 2024-04-18
### Added
- Print current time before starting waiting.
### Fixed
- Add a file-based lock to prevent concurrent grabbing of the same GPU.
### Changed
- Some texts in the program.
- Replace word "empty" with "idle".

## [0.1.0] - 2024-04-11

Initial release.