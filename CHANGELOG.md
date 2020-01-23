# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Types of changes**:
>
> - **Added**: for new features.
> - **Changed**: for changes in existing functionality.
> - **Deprecated**: for soon-to-be removed features.
> - **Removed**: for now removed features.
> - **Fixed**: for any bug fixes.
> - **Security**: in case of vulnerabilities.

## [Unreleased]

### Added

- [macos] get macos to compile
- [cpu][all] implement cpu_count and cpu_count_physical
- [cpu][macos] implement cpu_times, cpu_times_percent, and cpu_percent
- [disk] rename disk_io_counters_{perdisk,per_partition}
- [disk][unix] implement disk_usage
- [host][linux] implement boot_time
- [host] add Info
- [host][unix] implement Info
- [memory][macos] implement virtual_memory and swap_memory
- [network][macos] implement io counters
- [process] add ProcessCollector
- [process][unix] implement all signal methods
- [process][macos] implement Process::new
- [process][macos] implement process.name
- [process][macos] implement processes and pids
- [process][macos] implement Process.cpu_percent
- [process][macos] implement Process.cpu_times
- [process][macos] implement Process.memory_percent
- [process][macos] implement Process.memory_info
- [process][linux] implement pids
- [process][linux] implement pid_exists
- [process][linux] implement Process.cpu_percent
- [process][linux] implement Process.cpu_times
- [process][linux] implement Process.memory_percent
- [process][linux] implement Process.memory_info
- [process][linux] implement Process.uids
- [process][linux] implement Process.gids
- [process][linux] implement Process.send_signal
- [process][linux] implement Process.is_replaced
- [process][linux] implement Process.replace
- [process][linux] implement Process.parent
- [sensors][linux] implement temperatures

### Changed

- Overhaul the API
- Replace cpu_percent functions with CpuPercentCollector

### Removed

- Remove interval duration argument from various cpu percent functions
- Remove nowrap argument from collectors
- Remove reset method from collectors
- Remove inodes from DiskUsage
- Remove standalone CpuTimesPercent functions in favor of CpuTimesPercentCollector

## [1.7.0] - 2019-08-01

### Changed

- Remove `psutil::system` and replace with `psutil::{cpu, memory, host}`

### Removed

- Remove `getpid()`, `getppid()`, `Process.from_pidfile()`, `write_pidfile()`, and `read_pidfile()`

[Unreleased]: https://github.com/borntyping/rust-psutil/compare/v1.7.0...HEAD
[1.7.0]: https://github.com/borntyping/rust-psutil/compare/v1.6.0...v1.7.0
