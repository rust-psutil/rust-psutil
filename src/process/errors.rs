use std::io;

use snafu::Snafu;

use crate::{Error, Pid};

pub type ProcessResult<T> = std::result::Result<T, ProcessError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ProcessError {
	#[snafu(display("Process {} does not exists", pid))]
	NoSuchProcess { pid: Pid },

	#[snafu(display("Process {} is a zombie", pid))]
	ZombieProcess { pid: Pid },

	#[snafu(display("Access denied for process {}", pid))]
	AccessDenied { pid: Pid },

	#[snafu(display("psutil error for process {}: {}", pid, source))]
	PsutilError { pid: Pid, source: Error },
}

pub fn psutil_error_to_process_error(e: Error, pid: Pid) -> ProcessError {
	match e {
		Error::ReadFile { ref source, .. } => match source.kind() {
			io::ErrorKind::NotFound => ProcessError::NoSuchProcess { pid },
			io::ErrorKind::PermissionDenied => ProcessError::AccessDenied { pid },
			_ => ProcessError::PsutilError { pid, source: e },
		},
		_ => ProcessError::PsutilError { pid, source: e },
	}
}
