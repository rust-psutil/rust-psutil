use std::cmp;
use std::hash::{Hash, Hasher};
use std::io;
use std::mem;
use std::time::{Duration, Instant};

use crate::process::{pids, ProcessResult};
use crate::Pid;

#[derive(Clone, Debug)]
pub struct Process {
	pub(crate) pid: Pid,
	pub(crate) create_time: Duration,

	#[cfg(target_os = "linux")]
	pub(crate) busy: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) instant: Instant,
}

impl Process {
	pub fn current() -> ProcessResult<Process> {
		Process::new(std::process::id())
	}

	pub fn pid(&self) -> Pid {
		self.pid
	}

	/// The process creation time as a `Duration` since the boot time.
	/// The return value is different from Python psutil.
	pub fn create_time(&self) -> Duration {
		self.create_time
	}

	pub fn is_running(&self) -> bool {
		match Process::new(self.pid) {
			Ok(p) => p == *self,
			Err(_) => false,
		}
	}

	/// New method, not in Python psutil.
	pub fn is_replaced(&self) -> bool {
		match Process::new(self.pid) {
			Ok(p) => p != *self,
			Err(_) => false,
		}
	}

	/// New method, not in Python psutil.
	pub fn replace(&mut self) -> bool {
		match Process::new(self.pid) {
			Ok(p) => {
				if p == *self {
					false
				} else {
					mem::replace(self, p);
					true
				}
			}
			Err(_) => false,
		}
	}
}

impl PartialEq for Process {
	// Compares processes using their pid and create_time as a unique identifier.
	fn eq(&self, other: &Process) -> bool {
		(self.pid() == other.pid()) && (self.create_time() == other.create_time())
	}
}

impl cmp::Eq for Process {}

impl Hash for Process {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.pid().hash(state);
		self.create_time().hash(state);
	}
}

pub fn processes() -> io::Result<Vec<ProcessResult<Process>>> {
	let processes = pids()?.into_iter().map(Process::new).collect();

	Ok(processes)
}

#[cfg(test)]
mod unit_tests {
	use super::*;

	#[test]
	fn test_process_exe() {
		assert!(Process::current().unwrap().exe().is_ok());
	}

	#[test]
	fn test_process_cmdline() {
		assert!(Process::current().unwrap().cmdline().is_ok());
	}

	#[test]
	fn test_process_cwd() {
		assert!(Process::current().unwrap().cwd().is_ok());
	}

	#[test]
	fn test_process_equality() {
		assert_eq!(Process::current().unwrap(), Process::current().unwrap());
	}

	/// This could fail if you run the tests as PID 1. Please don't do that.
	#[test]
	fn test_process_inequality() {
		assert_ne!(Process::current().unwrap(), Process::new(1).unwrap());
	}

	#[test]
	fn test_processes() {
		processes().unwrap();
	}
}
