//! Read information about the operating system from `/proc`.

use std::str::FromStr;
use std::path::Path;

use utils::read_file;

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
