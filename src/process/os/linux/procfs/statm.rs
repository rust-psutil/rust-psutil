use std::fs;
use std::io;
use std::str::FromStr;

use crate::process::{io_error_to_process_error, procfs_path, ProcessResult};
use crate::utils::invalid_data;
use crate::{Pid, PAGE_SIZE};

/// Memory usage of a process read from `/proc/[pid]/statm`.
///
/// The `lib` [4, u64] and `dt` [6, u64] fields are ignored.
#[derive(Clone, Debug)]
pub struct ProcfsStatm {
	/// Total program size (bytes).
	pub size: u64,

	/// Resident Set Size (bytes).
	pub resident: u64,

	/// Shared pages (bytes).
	pub share: u64,

	/// Text.
	pub text: u64,

	/// Data + stack.
	pub data: u64,
}

impl FromStr for ProcfsStatm {
	type Err = io::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let fields: Vec<&str> = s.trim_end().split_whitespace().collect();

		if fields.len() != 7 {
			return Err(invalid_data(&format!(
				"Expected 7 fields, got {}",
				fields.len()
			)));
		}

		Ok(ProcfsStatm {
			size: try_parse!(fields[0], u64::from_str) * *PAGE_SIZE,
			resident: try_parse!(fields[1], u64::from_str) * *PAGE_SIZE,
			share: try_parse!(fields[2], u64::from_str) * *PAGE_SIZE,
			text: try_parse!(fields[3], u64::from_str) * *PAGE_SIZE,
			data: try_parse!(fields[5], u64::from_str) * *PAGE_SIZE,
		})
	}
}

pub fn procfs_statm(pid: Pid) -> ProcessResult<ProcfsStatm> {
	let data = fs::read_to_string(procfs_path(pid, "statm"))
		.map_err(|e| io_error_to_process_error(e, pid))?;

	ProcfsStatm::from_str(&data).map_err(|e| io_error_to_process_error(e, pid))
}
