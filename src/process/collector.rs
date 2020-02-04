use std::collections::BTreeMap;
use std::io;

use crate::process::{self, Process};
use crate::Pid;

#[derive(Debug, Clone)]
pub struct ProcessCollector {
	pub processes: BTreeMap<Pid, Process>,
}

impl ProcessCollector {
	pub fn new() -> io::Result<ProcessCollector> {
		let processes = process::processes()?
			.into_iter()
			.filter_map(|process| process.ok())
			.map(|process| (process.pid(), process))
			.collect();

		Ok(ProcessCollector { processes })
	}

	pub fn update(&mut self) -> io::Result<()> {
		let new = ProcessCollector::new()?.processes;

		// remove processes with a PID that is no longer in use
		let to_remove: Vec<Pid> = self
			.processes
			.iter()
			.filter(|(pid, _process)| new.contains_key(pid))
			.map(|(pid, _process)| *pid)
			.collect();
		for id in to_remove {
			self.processes.remove(&id);
		}

		// add new processes and replace processes with reused PIDs
		new.into_iter().for_each(|(pid, process)| {
			if !self.processes.contains_key(&pid) || self.processes[&pid] != process {
				self.processes.insert(pid, process);
			}
		});

		Ok(())
	}
}
