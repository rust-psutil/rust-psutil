# rust-psutil

[![crates.io](https://img.shields.io/crates/v/psutil.svg)](https://crates.io/crates/psutil)
[![docs.rs](https://docs.rs/psutil/badge.svg)](https://docs.rs/psutil)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.39+-green.svg)
[![Matrix](https://img.shields.io/badge/matrix-%23rust--psutil-blue.svg)](https://matrix.to/#/#rust-psutil:matrix.org)

A process and system monitoring library for Rust, heavily inspired by the [psutil] module for Python.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
psutil = "3.1.0"
```

Or to opt into only a certain submodule:

```toml
[dependencies]
psutil = { version = "3.1.0", default-features = false, features = ["process"] }
```

## Platform Support

The currently supported platforms include:
- Linux
- macOS
- Windows

[platform-support](./platform-support.md) details the implementation level of each platform.

## Apps using rust-psutil

- [procrec](https://github.com/gh0st42/procrec)

## Related projects

- [gopsutil](https://github.com/shirou/gopsutil)
- [hiem](https://github.com/heim-rs/heim)
- [psutil]
- [rust-battery](https://github.com/svartalf/rust-battery)
- [sys-info-rs](https://github.com/FillZpp/sys-info-rs)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [systemstat](https://github.com/myfreeweb/systemstat)

[psutil]: https://github.com/giampaolo/psutil
