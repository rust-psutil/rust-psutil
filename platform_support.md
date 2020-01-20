## Linux

### CPU

- [x] [cpu_times](https://psutil.readthedocs.io/en/latest/#psutil.cpu_times)
- [x] [cpu_percent](https://psutil.readthedocs.io/en/latest/#psutil.cpu_percent)
- [x] [cpu_times_percent](https://psutil.readthedocs.io/en/latest/#psutil.cpu_times_percent)
- [x] [cpu_count](https://psutil.readthedocs.io/en/latest/#psutil.cpu_count)
- [ ] [cpu_stats](https://psutil.readthedocs.io/en/latest/#psutil.cpu_stats)
- [ ] [cpu_freq](https://psutil.readthedocs.io/en/latest/#psutil.cpu_freq)

### Disk

- [x] [disk_partitions](https://psutil.readthedocs.io/en/latest/#psutil.disk_partitions)
- [x] [disk_usage](https://psutil.readthedocs.io/en/latest/#psutil.disk_usage)
- [x] [disk_io_counters](https://psutil.readthedocs.io/en/latest/#psutil.disk_io_counters)

### Host

- [x] [loadavg](https://psutil.readthedocs.io/en/latest/?badge=latest#psutil.getloadavg)
- [x] [boot_time](https://psutil.readthedocs.io/en/latest/#psutil.boot_time)
- [ ] [users](https://psutil.readthedocs.io/en/latest/#psutil.users)

### Memory

- [x] [virtual_memory](https://psutil.readthedocs.io/en/latest/#psutil.virtual_memory)
- [x] [swap_memory](https://psutil.readthedocs.io/en/latest/#psutil.swap_memory)

### Network

- [x] [net_io_counters](https://psutil.readthedocs.io/en/latest/#psutil.net_io_counters)
- [ ] [net_connections](https://psutil.readthedocs.io/en/latest/#psutil.net_connections)
- [ ] [net_if_addrs](https://psutil.readthedocs.io/en/latest/#psutil.net_if_addrs)
- [ ] [net_if_stats](https://psutil.readthedocs.io/en/latest/#psutil.net_if_stats)

### Processes

- [x] [pids](https://psutil.readthedocs.io/en/latest/#psutil.pids)
- [x] [process_iter](https://psutil.readthedocs.io/en/latest/#psutil.process_iter)
- [x] [pid_exists](https://psutil.readthedocs.io/en/latest/#psutil.pid_exists)
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
- [x] [parent](https://psutil.readthedocs.io/en/latest/#psutil.Process.parent)
- [ ] [parents](https://psutil.readthedocs.io/en/latest/#psutil.Process.parents)
- [x] [status](https://psutil.readthedocs.io/en/latest/#psutil.Process.status)
- [x] [cwd](https://psutil.readthedocs.io/en/latest/#psutil.Process.cwd)
- [ ] [username](https://psutil.readthedocs.io/en/latest/#psutil.Process.username)
- [ ] [uids](https://psutil.readthedocs.io/en/latest/#psutil.Process.uids)
- [ ] [gids](https://psutil.readthedocs.io/en/latest/#psutil.Process.gids)
- [ ] [terminal](https://psutil.readthedocs.io/en/latest/#psutil.Process.terminal)
- [ ] [nice](https://psutil.readthedocs.io/en/latest/#psutil.Process.nice)
- [ ] [ionice](https://psutil.readthedocs.io/en/latest/#psutil.Process.ionice)
- [ ] [rlimit](https://psutil.readthedocs.io/en/latest/#psutil.Process.rlimit)
- [ ] [io_counters](https://psutil.readthedocs.io/en/latest/#psutil.Process.io_counters)
- [ ] [num_ctx_switches](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_ctx_switches)
- [ ] [num_fds](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_fds)
- [ ] [num_threads](https://psutil.readthedocs.io/en/latest/#psutil.Process.num_threads)
- [ ] [threads](https://psutil.readthedocs.io/en/latest/#psutil.Process.threads)
- [x] [cpu_times](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_times)
- [x] [cpu_percent](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_percent)
- [ ] [cpu_affinity](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_affinity)
- [ ] [cpu_num](https://psutil.readthedocs.io/en/latest/#psutil.Process.cpu_num)
- [ ] [memory_info](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_info)
- [ ] [memory_info_full](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_info_full)
- [x] [memory_percent](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_percent)
- [ ] [memory_maps](https://psutil.readthedocs.io/en/latest/#psutil.Process.memory_maps)
- [ ] [children](https://psutil.readthedocs.io/en/latest/#psutil.Process.children)
- [x] [open_files](https://psutil.readthedocs.io/en/latest/#psutil.Process.open_files)
- [ ] [connections](https://psutil.readthedocs.io/en/latest/#psutil.Process.connections)
- [x] [is_running](https://psutil.readthedocs.io/en/latest/#psutil.Process.is_running)
- [x] [send_signal](https://psutil.readthedocs.io/en/latest/#psutil.Process.send_signal)
- [ ] [suspend](https://psutil.readthedocs.io/en/latest/#psutil.Process.suspend)
- [ ] [resume](https://psutil.readthedocs.io/en/latest/#psutil.Process.resume)
- [ ] [terminate](https://psutil.readthedocs.io/en/latest/#psutil.Process.terminate)
- [x] [kill](https://psutil.readthedocs.io/en/latest/#psutil.Process.kill)
- [ ] [wait](https://psutil.readthedocs.io/en/latest/#psutil.Process.wait)

### Sensors

- [x] [sensors_temperatures](https://psutil.readthedocs.io/en/latest/#psutil.sensors_temperatures)
- [ ] [sensors_fans](https://psutil.readthedocs.io/en/latest/#psutil.sensors_fans)
