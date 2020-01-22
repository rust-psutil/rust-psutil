use std::io;

use crate::process::processes;
use crate::Pid;

pub fn pids() -> io::Result<Vec<Pid>> {
	Ok(processes()?
		.into_iter()
		.filter_map(|process| process.is_ok())
		.map(|process| process.pid())
		.collect())
}

pub fn pid_exists(_pid: Pid) -> bool {
	todo!()
}
