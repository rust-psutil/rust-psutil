# Contributing to psutil

Thanks for contributing! Contribution is through standard Github mechanisms:

- to report bugs or ask questions (including about the contribution process) raise a Github issue
- to submit changes, create a [pull request](https://help.github.com/articles/creating-a-pull-request/). We'll try and give feedback on these fairly quickly.

## Features needed

psutil should be a fairly easy project to contribute to, as there's already [a reference implementation in Python](https://psutil.readthedocs.io/en/latest/) and we still need lots of contributions to get feature-parity in the Rust library.

If you'd like to make a contribution, try:

- picking an unimplemented function from the list below
- checking how it's implemented in the Python version of `psutil` (often by reading a file from `/proc`)
- porting that behaviour to this library, along with tests and documentation
- sending a pull request

### System-wide CPU

- [x] [cpu_times](https://psutil.readthedocs.io/en/latest/#psutil.cpu_times)
- [x] [cpu_percent](https://psutil.readthedocs.io/en/latest/#psutil.cpu_percent)
- [x] [cpu_times](https://psutil.readthedocs.io/en/latest/#psutil.cpu_times_percent)
- [x] [cpu_count](https://psutil.readthedocs.io/en/latest/#psutil.cpu_count)
- [ ] [cpu_stats](https://psutil.readthedocs.io/en/latest/#psutil.cpu_stats)
- [ ] [cpu_freq](https://psutil.readthedocs.io/en/latest/#psutil.cpu_freq)

### System-wide memory

- [x] [virtual_memory](https://psutil.readthedocs.io/en/latest/#psutil.virtual_memory)
- [x] [swap_memory](https://psutil.readthedocs.io/en/latest/#psutil.swap_memory)

### System-wide disk

- [x] [disk_partitions](https://psutil.readthedocs.io/en/latest/#psutil.disk_partitions)
- [x] [disk_usage](https://psutil.readthedocs.io/en/latest/#psutil.disk_usage)
- [x] [disk_io_counters](https://psutil.readthedocs.io/en/latest/#psutil.disk_io_counters)

### System-wide networking

- [x] [net_io_counters](https://psutil.readthedocs.io/en/latest/#psutil.net_io_counters)
- [ ] [net_connections](https://psutil.readthedocs.io/en/latest/#psutil.net_connections)
- [ ] [net_if_addrs](https://psutil.readthedocs.io/en/latest/#psutil.net_if_addrs)
- [ ] [net_if_stats](https://psutil.readthedocs.io/en/latest/#psutil.net_if_stats)

### System-wide sensors

- [ ] [sensors_temperatures](https://psutil.readthedocs.io/en/latest/#psutil.sensors_temperatures)
- [ ] [sensors_fans](https://psutil.readthedocs.io/en/latest/#psutil.sensors_fans)
- [ ] [sensors_battery](https://psutil.readthedocs.io/en/latest/#psutil.sensors_battery)

### System-wide (other)

- [ ] [boot_time](https://psutil.readthedocs.io/en/latest/#psutil.boot_time)
- [ ] [users](https://psutil.readthedocs.io/en/latest/#psutil.users)

### System-wide processes

- [ ] [pids](https://psutil.readthedocs.io/en/latest/#psutil.pids)
- [x] [process_iter](https://psutil.readthedocs.io/en/latest/#psutil.process_iter)
- [ ] [pid_exists](https://psutil.readthedocs.io/en/latest/#psutil.pid_exists)
- [ ] [wait_procs](https://psutil.readthedocs.io/en/latest/#psutil.wait_procs)

### Per-process

- [x] [pid](https://psutil.readthedocs.io/en/latest/#psutil.Process.pid)
- [x] [ppid](https://psutil.readthedocs.io/en/latest/#psutil.Process.ppid)
- [x] [name](https://psutil.readthedocs.io/en/latest/#psutil.Process.name)
- [x] [exe](https://psutil.readthedocs.io/en/latest/#psutil.Process.exe)
- [x] [cmdline](https://psutil.readthedocs.io/en/latest/#psutil.Process.cmdline)
- [x] [environ](https://psutil.readthedocs.io/en/latest/#psutil.Process.environ)
- [x] [create_time](https://psutil.readthedocs.io/en/latest/#psutil.Process.create_time)
- [ ] [as_dict](https://psutil.readthedocs.io/en/latest/#psutil.Process.as_dict)
- [ ] [parent](https://psutil.readthedocs.io/en/latest/#psutil.Process.parent)
- [ ] [status](https://psutil.readthedocs.io/en/latest/#psutil.Process.status)
- [x] [cwd](https://psutil.readthedocs.io/en/latest/#psutil.Process.cwd)
- [ ] [username](https://psutil.readthedocs.io/en/latest/#psutil.Process.username)
- [ ] [uids](https://psutil.readthedocs.io/en/latest/#psutil.Process.uids)
- [ ] [gids](https://psutil.readthedocs.io/en/latest/#psutil.Process.gids)
- [x] [terminal](https://psutil.readthedocs.io/en/latest/#psutil.Process.terminal)
- [x] [nice](https://psutil.readthedocs.io/en/latest/#psutil.Process.nice)
- [ ] [ionice](https://psutil.readthedocs.io/en/latest/#psutil.Process.ionice)
- [ ] [rlimit](https://psutil.readthedocs.io/en/latest/#psutil.Process.rlimit)
- [ ] [io_counters](https://psutil.readthedocs.io/en/latest/#psutil.Process.io_counters)
- [ ] [num_ctx_switches](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_ctx_switches)
- [ ] [num_fds](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_fds)
- [x] [num_threads](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_threads)
- [ ] [threads](https://psutil.readthedocs.io/en/latest/#psutil.Process.threads)
- [x] [cpu_times](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_times)
- [ ] [cpu_percent](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_percent)
- [ ] [cpu_affinity](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_affinity)
- [x] [cpu_num](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_num)
- [x] [memory_info](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_info)
- [ ] [memory_info_full](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_info_full)
- [ ] [memory_percent](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_percent)
- [ ] [memory_maps](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_maps)
- [ ] [children](https://psutil.readthedocs.io/en/latest/#psutil.Process.children)
- [x] [open_files](https://psutil.readthedocs.io/en/latest/#psutil.Process.open_files)
- [ ] [connections](https://psutil.readthedocs.io/en/latest/#psutil.Process.connections)
- [x] [is_running](https://psutil.readthedocs.io/en/latest/#psutil.Process.is_running)
- [ ] [send_signal](https://psutil.readthedocs.io/en/latest/#psutil.Process.send_signal)
- [ ] [suspend](https://psutil.readthedocs.io/en/latest/#psutil.Process.suspend)
- [ ] [resume](https://psutil.readthedocs.io/en/latest/#psutil.Process.resume)
- [ ] [terminate](https://psutil.readthedocs.io/en/latest/#psutil.Process.terminate)
- [x] [kill](https://psutil.readthedocs.io/en/latest/#psutil.Process.kill)
- [ ] [wait](https://psutil.readthedocs.io/en/latest/#psutil.Process.wait)

## Infrastructure improvements

Besides adding new features to the code, it would be nice to do the following:

- enhance `.travis.yml` to run `cargo fmt` against each pull request
- enhance `.travis.yml` to run `cargo clippy` against each pull request
- enhance `.travis.yml` to run <https://crates.io/crates/cargo-tarpaulin> for code coverage
- integrate with <coveralls.io> (via <https://github.com/xd009642/coveralls-api>)
- enhance `.travis.yml` to automatically `cargo publish` a crate when a new tag is pushed

## Releasing

When we're ready to release, a project owner should do the following
- Determine what the next version is, according to semver
- Update the version in `Cargo.toml`
- Tag the commit via `git tag -a v<X>.<Y>.<Z>`
- `git push upstream master --tag v<X>.<Y>.<Z>`
- Run `cargo publish` (run `cargo login` first if needed)
