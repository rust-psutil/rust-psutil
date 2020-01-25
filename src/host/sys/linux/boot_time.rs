use std::fs;
use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::utils::invalid_data;

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
