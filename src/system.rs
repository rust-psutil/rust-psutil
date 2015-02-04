//! Information about the system

use std::old_io::fs::File;
use std::str::FromStr;
use std::str::StrExt;

/// Returns the system uptime in seconds
///
/// `/proc/uptime` contains the system uptime and idle time
pub fn uptime() -> isize {
    let data = File::open(&Path::new("/proc/uptime")).read_to_string().unwrap();
    let numbers: Vec<&str> = data.split(' ').collect();
    let uptime: Vec<&str> = numbers[0].split('.').collect();
    return FromStr::from_str(uptime[0]).unwrap();
}
