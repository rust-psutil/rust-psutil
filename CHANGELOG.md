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

### Changed

- Completely overhaul the entire API

## [1.7.0] - 2019-08-01

### Removed

- Remove `getpid()`, `getppid()`, `Process.from_pidfile()`, `write_pidfile()`, and `read_pidfile()`
- Remove `psutil::system` and replace with `psutil::{cpu, memory, host}`

[Unreleased]: https://github.com/borntyping/rust-psutil/compare/v1.7.0...HEAD
[1.7.0]: https://github.com/borntyping/rust-psutil/compare/v1.6.0...v1.7.0
