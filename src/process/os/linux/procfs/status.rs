use std::fs;
use std::io;
use std::str::FromStr;

use crate::process::{io_error_to_process_error, procfs_path, ProcessResult};
use crate::utils::invalid_data;
use crate::Pid;

// TODO: rest of the fields
/// New struct, not in Python psutil
#[derive(Clone, Debug)]
pub struct ProcfsStatus {
	/// Voluntary context switches.
	pub voluntary_ctxt_switches: u64,

	/// Non-voluntary context switches.
	pub nonvoluntary_ctxt_switches: u64,
}

impl FromStr for ProcfsStatus {
	type Err = io::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let lines = s
			.lines()
			.map(|line| {
				line.split_whitespace()
					.collect::<Vec<&str>>()
					.get(1)
					.copied()
					.unwrap_or_default()
			})
			.collect::<Vec<&str>>();

		if lines.len() != 55 {
			return Err(invalid_data(&format!(
				"Expected 55 lines, got {}",
				lines.len()
			)));
		}

		let voluntary_ctxt_switches = try_parse!(lines[53]);
		let nonvoluntary_ctxt_switches = try_parse!(lines[54]);

		Ok(ProcfsStatus {
			voluntary_ctxt_switches,
			nonvoluntary_ctxt_switches,
		})
	}
}

/// New function, not in Python psutil
pub fn procfs_status(pid: Pid) -> ProcessResult<ProcfsStatus> {
	let data = fs::read_to_string(procfs_path(pid, "status"))
		.map_err(|e| io_error_to_process_error(e, pid))?;

	ProcfsStatus::from_str(&data).map_err(|e| io_error_to_process_error(e, pid))
}
