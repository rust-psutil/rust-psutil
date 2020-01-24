use std::collections::HashMap;
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
	pub voluntary_ctxt_switches: Option<u64>,

	/// Non-voluntary context switches.
	pub nonvoluntary_ctxt_switches: Option<u64>,
}

impl FromStr for ProcfsStatus {
	type Err = io::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let map = s
			.lines()
			.map(|line| {
				let fields = line.splitn(2, ':').collect::<Vec<&str>>();
				if fields.len() != 2 {
					return Err(invalid_data(&format!(
						"Expected 2 fields, got {}",
						fields.len()
					)));
				}
				Ok((fields[0], fields[1].trim()))
			})
			.collect::<io::Result<HashMap<&str, &str>>>()?;

		let uid_fields: Vec<&str> = map
			.get("Uid")
			.ok_or_else(|| invalid_data("Missing Uid"))?
			.split_whitespace()
			.collect();
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

		let gid_fields: Vec<&str> = map
			.get("Gid")
			.ok_or_else(|| invalid_data("Missing Gid"))?
			.split_whitespace()
			.collect();
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

		let voluntary_ctxt_switches = map
			.get("voluntary_ctxt_switches")
			.map(|entry| -> io::Result<u64> { Ok(try_parse!(entry)) })
			.transpose()?;
		let nonvoluntary_ctxt_switches = map
			.get("nonvoluntary_ctxt_switches")
			.map(|entry| -> io::Result<u64> { Ok(try_parse!(entry)) })
			.transpose()?;

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
