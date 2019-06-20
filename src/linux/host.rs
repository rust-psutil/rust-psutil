use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

use crate::PID;

#[derive(Debug)]
pub struct LoadAverage {
    /// number of jobs in the run queue averaged over 1 minute
    pub one: f32,

    /// number of jobs in the run queue averaged over 5 minute
    pub five: f32,

    /// number of jobs in the run queue averaged over 15 minute
    pub fifteen: f32,

    /// current number of runnable kernel entities
    pub runnable: i32,

    /// total number of runnable kernel entities
    pub total_runnable: i32,

    /// pid for the most recently created process
    pub last_pid: PID,
}

/// Returns the system uptime in seconds.
///
/// `/proc/uptime` contains the system uptime and idle time.
pub fn uptime() -> isize {
    let data = fs::read_to_string("/proc/uptime").unwrap();
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

/// Returns the system load average
///
/// `/proc/loadavg` contains the system load average
pub fn loadavg() -> Result<LoadAverage> {
    let data = fs::read_to_string("/proc/loadavg")?;
    loadavg_internal(&data)
}

fn loadavg_internal(data: &str) -> Result<LoadAverage> {
    // strips off any trailing new line as well
    let fields: Vec<&str> = data.split_whitespace().collect();

    let one = try_parse!(fields[0]);
    let five = try_parse!(fields[1]);
    let fifteen = try_parse!(fields[2]);
    let last_pid = try_parse!(fields[4]);

    let entities = fields[3].split('/').collect::<Vec<&str>>();
    let runnable = try_parse!(entities[0]);
    let total_runnable = try_parse!(entities[1]);

    Ok(LoadAverage {
        one,
        five,
        fifteen,
        runnable,
        total_runnable,
        last_pid,
    })
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn uptime_parses() {
        assert_eq!(uptime_internal("12489513.08 22906637.29\n"), 12_489_513);
    }

    #[test]
    fn loadavg_parses() {
        let input = "0.49 0.70 0.84 2/519 1454\n";
        let out = loadavg_internal(input).unwrap();
        assert_eq!(out.one, 0.49);
        assert_eq!(out.five, 0.70);
        assert_eq!(out.fifteen, 0.84);
        assert_eq!(out.total_runnable, 519);
        assert_eq!(out.runnable, 2);
        assert_eq!(out.last_pid, 1454);
    }
}
