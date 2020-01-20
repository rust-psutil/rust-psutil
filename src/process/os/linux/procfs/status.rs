use std::fs;
use std::io;
use std::str::FromStr;

use crate::process::os::unix::{Gid, Uid};
use crate::process::{io_error_to_process_error, procfs_path, ProcessResult};
use crate::utils::invalid_data;
use crate::Pid;

// TODO: rest of the fields
/// New struct, not in Python psutil.
#[derive(Clone, Debug)]
pub struct ProcfsStatus {
	pub uid: [Uid; 4],

	pub gid: [Gid; 4],

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
				line.splitn(2, '\t')
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

		let uid_fields: Vec<&str> = lines[8].split_whitespace().collect();
		if uid_fields.len() != 4 {
			return Err(invalid_data(&format!(
				"Expected 4 fields, got {}",
				uid_fields.len()
			)));
		}
		let uid = [
			try_parse!(uid_fields[0]),
			try_parse!(uid_fields[1]),
			try_parse!(uid_fields[2]),
			try_parse!(uid_fields[3]),
		];

		let gid_fields: Vec<&str> = lines[9].split_whitespace().collect();
		if gid_fields.len() != 4 {
			return Err(invalid_data(&format!(
				"Expected 4 fields, got {}",
				gid_fields.len()
			)));
		}
		let gid = [
			try_parse!(gid_fields[0]),
			try_parse!(gid_fields[1]),
			try_parse!(gid_fields[2]),
			try_parse!(gid_fields[3]),
		];

		let voluntary_ctxt_switches = try_parse!(lines[53]);
		let nonvoluntary_ctxt_switches = try_parse!(lines[54]);

		Ok(ProcfsStatus {
			uid,
			gid,
			voluntary_ctxt_switches,
			nonvoluntary_ctxt_switches,
		})
	}
}

/// New function, not in Python psutil.
pub fn procfs_status(pid: Pid) -> ProcessResult<ProcfsStatus> {
	let data = fs::read_to_string(procfs_path(pid, "status"))
		.map_err(|e| io_error_to_process_error(e, pid))?;

	ProcfsStatus::from_str(&data).map_err(|e| io_error_to_process_error(e, pid))
}
