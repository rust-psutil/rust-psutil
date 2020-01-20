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
		// remove dead processes
		let to_remove: Vec<Pid> = self
			.processes
			.iter()
			.filter(|(_pid, process)| process.is_running())
			.map(|(pid, _process)| *pid)
			.collect();
		for id in to_remove {
			self.processes.remove(&id);
		}

		// add new processes and replace processes with reused PIDs
		process::processes()?
			.into_iter()
			.filter_map(|process| process.ok())
			.for_each(|process| {
				if !self.processes.contains_key(&process.pid())
					|| self.processes[&process.pid()] != process
				{
					self.processes.insert(process.pid(), process);
				}
			});

		Ok(())
	}
}
