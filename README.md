# rust-psutil

[![Latest Version](https://img.shields.io/crates/v/psutil.svg)](https://crates.io/crates/psutil)
[![Latest Version](https://docs.rs/psutil/badge.svg)](https://docs.rs/psutil)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.39+-green.svg)

A process and system monitoring library for Rust, heavily inspired by the [psutil] module for Python.

## Platform Support

Currently, only Linux is supported, but support is planned for all major platforms.

[platform_support](./platform_support.md) details the implementation level of each platform.

## License

`rust-psutil` is licensed under the [MIT License].

## Authors

Originally written by [Sam Clements], maintained by [Sam Clements], [Rob Day], and [Caleb Bassi], and developed by multiple [contributors].

## Related projects

`rust-psutil` has no connection to these projects, but you might find them useful.

- [hiem](https://github.com/heim-rs/heim)
- [rust-battery](https://github.com/svartalf/rust-battery)
- [systemstat](https://github.com/myfreeweb/systemstat)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)

[MIT License]: https://opensource.org/licenses/MIT
[psutil]: https://github.com/giampaolo/psutil
[Sam Clements]: https://github.com/borntyping
[Rob Day]: https://github.com/rkday
[Caleb Bassi]: https://github.com/cjbassi
[contributors]: https://github.com/borntyping/rust-psutil/graphs/contributors
