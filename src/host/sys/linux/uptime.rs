use std::fs;
use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::utils::invalid_data;

fn parse_uptime(data: &str) -> io::Result<Duration> {
    let fields: Vec<&str> = data.split_whitespace().collect();
    if fields.len() != 2 {
        return Err(invalid_data(&format!("malformed uptime data: '{}'", data)));
    }
    let uptime: Vec<&str> = fields[0].split('.').collect();
    if uptime.len() != 2 {
        return Err(invalid_data(&format!("malformed uptime data: '{}'", data)));
    }
    let (seconds, centiseconds): (u64, u32) = (try_parse!(uptime[0]), try_parse!(uptime[1]));
    let uptime = Duration::new(seconds, centiseconds * 10_000_000);

    Ok(uptime)
}

pub fn uptime() -> io::Result<Duration> {
    let data = fs::read_to_string("/proc/uptime")?;

    parse_uptime(&data)
}

fn parse_boot_time(data: &str) -> io::Result<SystemTime> {
    for line in data.lines() {
        if line.starts_with("btime ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(invalid_data(&format!(
                    "malformed '/proc/stat' data: '{}'",
                    data
                )));
            }
            let boot_time = UNIX_EPOCH + Duration::from_secs(try_parse!(parts[1]));

            return Ok(boot_time);
        }
    }

    Err(invalid_data(&format!(
        "malformed '/proc/stat' data: '{}'",
        data
    )))
}

// TODO: cache with https://github.com/jaemk/cached once `pub fn` is supported
pub fn boot_time() -> io::Result<SystemTime> {
    let data = fs::read_to_string("/proc/stat")?;
    let boot_time = parse_boot_time(&data)?;

    Ok(boot_time)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_uptime() {
        assert!(uptime().unwrap().as_secs() > 0);
    }

    #[test]
    fn test_parse_uptime() {
        assert_eq!(
            parse_uptime("12489513.08 22906637.29\n").unwrap(),
            Duration::new(12_489_513, 8 * 10_000_000)
        );
    }
}
