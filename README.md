# rust-psutil

[![Latest Version](https://img.shields.io/crates/v/psutil.svg)](https://crates.io/crates/psutil)
[![Latest Version](https://docs.rs/psutil/badge.svg)](https://docs.rs/psutil)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.39+-green.svg)
[![Matrix](https://img.shields.io/matrix/rust-psutil:matrix.org)](https://matrix.to/#/#rust-psutil:matrix.org)

A process and system monitoring library for Rust, heavily inspired by the [psutil] module for Python.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
psutil = "3.0.0"
```

Or to opt into only a certain submodule:

```toml
[dependencies]
psutil = { version = "3.0.0", default-features = false, features = ["process"] }
```

## Platform Support

Currently, only Linux and macOS are supported, but support is planned for all major platforms.

[platform_support](./platform_support.md) details the implementation level of each platform.

## License

`rust-psutil` is licensed under the [MIT License].

## Authors

Originally written by [Sam Clements], maintained by [Sam Clements], [Rob Day], and [Caleb Bassi], and developed by multiple [contributors].

## Apps using rust-psutil

- [ytop](https://github.com/cjbassi/ytop)

## Related projects

- [hiem](https://github.com/heim-rs/heim)
- [rust-battery](https://github.com/svartalf/rust-battery)
- [systemstat](https://github.com/myfreeweb/systemstat)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [gopsutil](https://github.com/shirou/gopsutil)
- [psutil]
- [sys-info-rs](https://github.com/FillZpp/sys-info-rs)

[MIT License]: https://opensource.org/licenses/MIT
[psutil]: https://github.com/giampaolo/psutil
[Sam Clements]: https://github.com/borntyping
[Rob Day]: https://github.com/rkday
[Caleb Bassi]: https://github.com/cjbassi
[contributors]: https://github.com/borntyping/rust-psutil/graphs/contributors
