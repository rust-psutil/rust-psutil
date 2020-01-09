# rust-psutil [![](https://img.shields.io/github/tag/borntyping/rust-psutil.svg)](https://github.com/borntyping/rust-psutil/tags) [![](https://img.shields.io/github/issues/borntyping/rust-psutil.svg)](https://github.com/borntyping/rust-psutil/issues)

A process and system monitoring library for Rust, heavily inspired by the [psutil] module for Python.

Note that most functionality currently only works on Linux, but support is planned for all major platforms.

* [Source on GitHub](https://github.com/borntyping/rust-psutil)
* [Packages on Crates.io](https://crates.io/crates/psutil)

## Run examples

The examples can be run using `cargo run --example <name>`:

```bash
cargo run --example status
cargo run --example ps
...
```

## Licence

`rust-psutil` is licenced under the [MIT Licence].

## Authors

Originally written by [Sam Clements], maintained by [Sam Clements] and [Rob Day], and developed by multiple [contributors].

## Related projects

`rust-psutil` has no connection to these projects, but you might find them useful.

* [hiem](https://crates.io/crates/heim)
* [rust-battery](https://github.com/svartalf/rust-battery)

[MIT Licence]: http://opensource.org/licenses/MIT
[psutil]: https://github.com/giampaolo/psutil/
[Sam Clements]: https://github.com/borntyping
[Rob Day]: https://github.com/rkday
[contributors]: https://github.com/borntyping/rust-psutil/graphs/contributors
