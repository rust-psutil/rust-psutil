//! Read information about the operating system from `/proc`.

use std::str::FromStr;
use std::path::Path;
use std::collections::HashMap;

use utils::read_file;

/// Gets a value from a hashmap and dereferences it.
/// If the key doesn't exist, return `None`
macro_rules! get_deref(
    ($map:expr,$key:expr) => (
        match $map.get($key) {
            Some(e) => *e,
            None => return None
        }
    )
);

#[derive(Debug)]
pub struct VirtualMemory {
    /// Amount of total memory
    pub total: u64,

    /// Amount of memory available for new processes
    pub available: u64,

    /// Percent of memory used
    pub percent: f32,

    /// Memory currently in use
    pub used: u64,

    /// Memory not being used
    pub free: u64,

    /// Memory currently in use
    pub active: u64,

    /// Memory that is not in use
    pub inactive: u64,

    /// Temporary storage for raw disk blocks
    pub buffers: u64,

    /// Memory used by the page cache
    pub cached: u64,

    /// Amount of memory consumed by tmpfs filesystems
    pub shared: u64,
}

impl Default for VirtualMemory {
    fn default() -> VirtualMemory {
        VirtualMemory {
            total: 0,
            available: 0,
            percent: 0.0,
            used: 0,
            free: 0,
            active: 0,
            inactive: 0,
            buffers: 0,
            cached: 0,
            shared: 0,
        }
    }
}

#[derive(Debug)]
pub struct SwapMemory {
    /// Amount of total swap memory
    pub total: u64,

    /// Amount of used swap memory
    pub used: u64,

    /// Amount of free swap memory
    pub free: u64,

    /// Percent of sway memory used
    pub percent: f32,

    /// Amount of memory swapped in from disk
    pub sin: u64,

    /// Amount of memory swapped to disk
    pub sout: u64,
}

impl Default for SwapMemory {
    fn default() -> Self {
        SwapMemory {
            total: 0,
            used: 0,
            free: 0,
            percent: 0.0,
            sin: 0,
            sout: 0,
        }
    }
}

/// Returns the system uptime in seconds.
///
/// `/proc/uptime` contains the system uptime and idle time.
pub fn uptime() -> isize {
    let data = read_file(Path::new("/proc/uptime")).unwrap();
    uptime_internal(&data)
}

/// Returns the system uptime in seconds.
///
/// Input should be in the format '12489513.08 22906637.29\n'
fn uptime_internal(data: &str) -> isize {
    let numbers: Vec<&str> = data.split(' ').collect();
    let uptime: Vec<&str> = numbers[0].split('.').collect();
    FromStr::from_str(uptime[0]).unwrap()
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn uptime_parses() {
        assert_eq!(uptime_internal("12489513.08 22906637.29\n"), 12489513);
    }
}

/// Returns information about virtual memory usage
///
/// `/proc/meminfo` contains the virtual memory statistics
pub fn virtual_memory() -> Option<VirtualMemory> {
    let data = read_file(Path::new("/proc/meminfo")).unwrap();
    let mem_info = make_map(&data);

    let mut virtual_memory: VirtualMemory = Default::default();

    virtual_memory.total = get_deref!(mem_info, "MemTotal:");
    virtual_memory.free = get_deref!(mem_info, "MemFree:");
    virtual_memory.buffers = get_deref!(mem_info, "Buffers:");
    virtual_memory.cached = get_deref!(mem_info, "Cached:");
    virtual_memory.active = get_deref!(mem_info, "Active:");
    virtual_memory.inactive = get_deref!(mem_info, "Inactive:");

    // MemAvailable was introduced in kernel 3.14. The original psutil computes it if it's not
    // found, but since 3.14 has already reached EOL, let's assume that it's there.
    virtual_memory.available = get_deref!(mem_info, "MemAvailable:");

    // Shmem was introduced in 2.6.19
    virtual_memory.shared = get_deref!(mem_info, "Shmem:");

    virtual_memory.used = virtual_memory.total - virtual_memory.free - virtual_memory.cached -
        virtual_memory.buffers;
    virtual_memory.percent = (virtual_memory.used as f32 / virtual_memory.total as f32) * 100.0;

    Some(virtual_memory)
}

/// Returns information about swap memory usage
///
/// `/proc/meminfo` and `/proc/vmstat` contains the information
pub fn swap_memory() -> Option<SwapMemory> {
    let data = read_file(Path::new("/proc/meminfo")).unwrap();
    let swap_info = make_map(&data);

    let vmstat = read_file(Path::new("/proc/vmstat")).unwrap();
    let vmstat_info = make_map(&vmstat);

    let mut swap_memory: SwapMemory = Default::default();

    swap_memory.total = get_deref!(swap_info, "SwapTotal:");
    swap_memory.free = get_deref!(swap_info, "SwapFree:");
    swap_memory.sin = get_deref!(vmstat_info, "pswpin");
    swap_memory.sout = get_deref!(vmstat_info, "pswpout");
    swap_memory.used = swap_memory.total - swap_memory.free;
    swap_memory.percent = (swap_memory.used as f32 / swap_memory.total as f32) * 100.0;

    Some(swap_memory)
}

fn get_multiplier(fields: &mut Vec<&str>) -> Option<u64> {
    if let Some(ext) = fields.pop() {
        let multiplier = match ext {
            "kB" => Some(1024),
            _ => None,
        };
        fields.push(ext);

        multiplier
    } else {
        None
    }
}

fn make_map(data: &str) -> HashMap<&str, u64> {
    let lines: Vec<&str> = data.lines().collect();
    let mut map = HashMap::new();

    for line in lines {
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        let key = fields[0];
        let mut value = fields[1].parse::<u64>().unwrap();

        if let Some(multiplier) = get_multiplier(&mut fields) {
            value = value * multiplier;
        }

        map.insert(key, value);
    }

    map
}
