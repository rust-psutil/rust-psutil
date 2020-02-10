use std::time::Duration;

use snafu::{ensure, ResultExt};

use crate::{read_file, MissingData, ParseFloat, Result};

const PROC_UPTIME: &str = "/proc/uptime";

fn parse_uptime(contents: &str) -> Result<Duration> {
	let fields: Vec<&str> = contents.split_whitespace().collect();

	ensure!(
		fields.len() >= 2,
		MissingData {
			path: PROC_UPTIME,
			contents,
		}
	);

	let parsed = fields[0].parse().context(ParseFloat {
		path: PROC_UPTIME,
		contents,
	})?;
	let uptime = Duration::from_secs_f64(parsed);

	Ok(uptime)
}

/// New function, not in Python psutil.
pub fn uptime() -> Result<Duration> {
	parse_uptime(&read_file(PROC_UPTIME)?)
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
			Duration::from_secs_f64(12_489_513.08)
		);
	}
}
