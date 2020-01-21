use std::io;
use std::time::Duration;

use crate::process::ProcessResult;
use crate::Pid;

#[derive(Clone, Debug)]
pub struct Process {
	pub(crate) pid: Pid,
	pub(crate) create_time: Duration,
}

impl Process {
	pub fn pid(&self) -> Pid {
		self.pid
	}

	pub fn is_running(&self) -> bool {
		todo!()
	}
}

impl PartialEq for Process {
	// Compares processes using their pid and create_time as a unique identifier.
	fn eq(&self, other: &Process) -> bool {
		(self.pid == other.pid) && (self.create_time == other.create_time)
	}
}

pub fn processes() -> io::Result<Vec<ProcessResult<Process>>> {
	todo!()
}
