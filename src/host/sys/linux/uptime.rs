use std::fs;
use std::io;
use std::time::Duration;

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

/// New function, not in Python psutil.
pub fn uptime() -> io::Result<Duration> {
	let data = fs::read_to_string("/proc/uptime")?;

	parse_uptime(&data)
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
