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
psutil = "4.0.0"
```

Or to only use certain submodules:

```toml
[dependencies]
psutil = { version = "4.0.0", default-features = false, features = ["cpu", "process"] }
```

## Support

This project is not well maintained, and there are a host of other projects that may
meet your needs better. rust-psutil started as a university student's project before the
Rust 1.0 release (all the way back in January 2015!), and has had irregular maintenance
by a small group of developers since then.

There's no intent to archive it any time soon, as there are still projects using
rust-psutil and we get the occasional pull request to implement a new feature. Open a
discussion if you'd be interested in helping maintain it, ideally after you've made one
or two contributions.

See "Related Projects" below for a list of projects that overlap with rust-psutil's
functionality. Hiem has a comparison between Rust libraries with similar functionality:
https://github.com/heim-rs/heim/blob/master/COMPARISON.md.

### Versioning

The API is relatively unstable. The version number attempts to follow semantic
versioning, and you should major version bumps to include multiple breaking changes—most
significant new features added to rust-psutil required adjusting existing APIs.

### Platform Support

Currently, only Linux and macOS are supported at all.

[platform-support.md](./platform-support.md) details the implementation level of each platform.

### Related projects

_† Direct dependencies._

- Rust
  - [darwin-libproc](https://github.com/heim-rs/darwin-libproc)†
  - [hiem](https://github.com/heim-rs/heim)
  - [libproc-rs](https://github.com/andrewdavidmackenzie/libproc-rs)
  - [mach2](https://github.com/JohnTitor/mach2)†
  - [num_cpus](https://github.com/seanmonstar/num_cpus)†
  - [platforms](https://github.com/rustsec/rustsec)†
  - [procfs](https://github.com/eminence/procfs)
  - [rust-battery](https://github.com/svartalf/rust-battery)
  - [rust-users](https://github.com/ogham/rust-users)
  - [sys-info-rs](https://github.com/FillZpp/sys-info-rs)
  - [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
  - [systemstat](https://github.com/myfreeweb/systemstat)
- Golang
  - [gopsutil](https://github.com/shirou/gopsutil)
- Python
  - [psutil]

#### Apps using rust-psutil

- [procrec](https://github.com/gh0st42/procrec)

You can also see GitHub's [list of dependents](https://github.com/rust-psutil/rust-psutil/network/dependents).

## License

Released under the MIT license—see [LICENCE](./LICENSE).

[psutil]: https://github.com/giampaolo/psutil
