//! Read information about the operating system from `/proc`.

use std::str::FromStr;
use std::path::Path;

use utils::read_file;

/// Returns the system uptime in seconds.
///
/// `/proc/uptime` contains the system uptime and idle time.
pub fn uptime() -> isize {
    let data = read_file(&Path::new("/proc/uptime")).unwrap();
    let numbers: Vec<&str> = data.split(' ').collect();
    let uptime: Vec<&str> = numbers[0].split('.').collect();
    return FromStr::from_str(uptime[0]).unwrap();
}
