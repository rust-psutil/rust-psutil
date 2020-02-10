use std::time::{Duration, SystemTime, UNIX_EPOCH};

use snafu::{ensure, OptionExt, ResultExt};

use crate::{read_file, MissingData, ParseInt, Result};

const PROC_STAT: &str = "/proc/stat";

fn parse_boot_time(line: &str) -> Result<SystemTime> {
	let fields: Vec<&str> = line.split_whitespace().collect();

	ensure!(
		fields.len() >= 2,
		MissingData {
			path: PROC_STAT,
			contents: line,
		}
	);

	let parsed = fields[1].parse().context(ParseInt {
		path: PROC_STAT,
		contents: line,
	})?;
	let boot_time = UNIX_EPOCH + Duration::from_secs(parsed);

	Ok(boot_time)
}

// TODO: cache with https://github.com/jaemk/cached once `pub fn` is supported
pub fn boot_time() -> Result<SystemTime> {
	let contents = read_file(PROC_STAT)?;
	let line = contents
		.lines()
		.filter(|line| line.starts_with("btime "))
		.nth(0)
		.context(MissingData {
			path: PROC_STAT,
			contents: &contents,
		})?;

	parse_boot_time(line)
}
