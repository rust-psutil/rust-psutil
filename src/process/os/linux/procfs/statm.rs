use std::str::FromStr;

use snafu::{ensure, ResultExt};

use crate::process::{procfs_path, psutil_error_to_process_error, ProcessResult};
use crate::{read_file, Error, MissingData, ParseInt, Pid, Result, PAGE_SIZE};

const STATM: &str = "statm";

/// Memory usage of a process read from `/proc/[pid]/statm`.
///
/// The `lib` [4, u64] and `dt` [6, u64] fields are ignored.
/// New struct, not in Python psutil.
#[derive(Clone, Debug)]
pub struct ProcfsStatm {
	/// Total program size (bytes).
	pub size: u64,

	/// Resident Set Size (bytes).
	pub resident: u64,

	/// Shared pages (bytes).
	pub shared: u64,

	/// Text.
	pub text: u64,

	/// Data + stack.
	pub data: u64,
}

impl FromStr for ProcfsStatm {
	type Err = Error;

	fn from_str(contents: &str) -> Result<Self> {
		let fields: Vec<&str> = contents.trim_end().split_whitespace().collect();

		ensure!(
			fields.len() >= 7,
			MissingData {
				path: STATM,
				contents,
			}
		);

		let parse = |s: &str| -> Result<u64> {
			s.parse().context(ParseInt {
				path: STATM,
				contents,
			})
		};

		Ok(ProcfsStatm {
			size: parse(fields[0])? * *PAGE_SIZE,
			resident: parse(fields[1])? * *PAGE_SIZE,
			shared: parse(fields[2])? * *PAGE_SIZE,
			text: parse(fields[3])? * *PAGE_SIZE,
			data: parse(fields[5])? * *PAGE_SIZE,
		})
	}
}

/// New function, not in Python psutil.
pub fn procfs_statm(pid: Pid) -> ProcessResult<ProcfsStatm> {
	let contents =
		read_file(procfs_path(pid, STATM)).map_err(|e| psutil_error_to_process_error(e, pid))?;

	ProcfsStatm::from_str(&contents).map_err(|e| psutil_error_to_process_error(e, pid))
}
