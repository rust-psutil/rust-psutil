use std::fs;
use std::path::PathBuf;
use std::string::ToString;
use std::time::Instant;

use nix::sys::signal::{kill, Signal};
use nix::unistd;
use snafu::ResultExt;

use crate::common::NetConnectionType;
use crate::memory;
use crate::process::os::linux::{procfs_stat, ProcessExt as _};
use crate::process::{
	errors, io_error_to_process_error, MemType, OpenFile, Process, ProcessCpuTimes, ProcessError,
	ProcessResult, Status,
};
use crate::utils::calculate_cpu_percent;
use crate::{Count, Percent, Pid};

/// Returns a path to a file in `/proc/[pid]/`.
pub(crate) fn procfs_path(pid: Pid, name: &str) -> PathBuf {
	PathBuf::from("/proc").join(pid.to_string()).join(&name)
}

impl Process {
	pub fn new(pid: Pid) -> ProcessResult<Process> {
		let procfs_stat = procfs_stat(pid)?;
		let create_time = procfs_stat.starttime;
		let busy = ProcessCpuTimes::from(procfs_stat).busy();
		let instant = Instant::now();

		Ok(Process {
			pid,
			create_time,
			busy,
			instant,
		})
	}

	pub(crate) fn procfs_path(&self, name: &str) -> PathBuf {
		procfs_path(self.pid, name)
	}

	pub fn ppid(&self) -> ProcessResult<Option<Pid>> {
		Ok(self.procfs_stat()?.ppid)
	}

	pub fn name(&self) -> ProcessResult<String> {
		Ok(self.procfs_stat()?.comm)
	}

	pub fn exe(&self) -> ProcessResult<PathBuf> {
		fs::read_link(self.procfs_path("exe")).map_err(|e| io_error_to_process_error(e, self.pid))
	}

	/// On Linux, an `Ok(None)` is usually due to the process being a kernel thread.
	/// The return value is different from Python psutil.
	pub fn cmdline(&self) -> ProcessResult<Option<String>> {
		Ok(self.cmdline_vec()?.map(|c| c.join(" ")))
	}

	/// New method, not in Python psutil.
	/// On Linux, an `Ok(None)` is usually due to the process being a kernel thread.
	pub fn cmdline_vec(&self) -> ProcessResult<Option<Vec<String>>> {
		let cmdline = fs::read_to_string(&self.procfs_path("cmdline"))
			.map_err(|e| io_error_to_process_error(e, self.pid))?;

		if cmdline.is_empty() {
			return Ok(None);
		}

		let split = cmdline
			.split_terminator('\0')
			.map(|x| x.to_string())
			.collect();

		Ok(Some(split))
	}

	/// Preemptively checks if the process is still alive.
	pub fn parent(&self) -> ProcessResult<Option<Process>> {
		if !self.is_running() {
			return Err(ProcessError::NoSuchProcess { pid: self.pid });
		}

		let ppid = self.ppid()?;
		match ppid {
			Some(ppid) => Ok(Some(Process::new(ppid)?)),
			None => Ok(None),
		}
	}

	pub fn parents(&self) -> Option<Vec<Process>> {
		todo!()
	}

	pub fn status(&self) -> ProcessResult<Status> {
		Ok(self.procfs_stat()?.state)
	}

	pub fn cwd(&self) -> ProcessResult<PathBuf> {
		fs::read_link(self.procfs_path("cwd")).map_err(|e| io_error_to_process_error(e, self.pid))
	}

	pub fn username(&self) -> String {
		todo!()
	}

	pub fn get_nice(&self) -> i32 {
		todo!()
	}

	pub fn set_nice(&self, _nice: i32) {
		todo!()
	}

	pub fn num_ctx_switches(&self) -> Count {
		todo!()
	}

	pub fn num_threads(&self) -> Count {
		todo!()
	}

	pub fn threads(&self) {
		todo!()
	}

	pub fn cpu_times(&self) -> ProcessResult<ProcessCpuTimes> {
		let stat = self.procfs_stat()?;

		Ok(ProcessCpuTimes::from(stat))
	}

	/// Returns the cpu percent since the process was created or since the last time this method was
	/// called.
	/// Differs from Python psutil since there is no interval argument.
	pub fn cpu_percent(&mut self) -> ProcessResult<Percent> {
		let busy = self.cpu_times()?.busy();
		let instant = Instant::now();
		let percent = calculate_cpu_percent(self.busy, busy, instant - self.instant);
		self.busy = busy;
		self.instant = instant;

		Ok(percent)
	}

	pub fn memory_info(&self) {
		todo!()
	}

	pub fn memory_full_info(&self) {
		todo!()
	}

	// TODO: memtype argument
	pub fn memory_percent(&self) -> ProcessResult<Percent> {
		let statm = self.procfs_statm()?;
		let virtual_memory =
			memory::virtual_memory().map_err(|e| io_error_to_process_error(e, self.pid))?;
		let percent = ((statm.resident as f64 / virtual_memory.total as f64) * 100.0) as f32;

		Ok(percent)
	}

	pub fn memory_percent_with_type(&self, _type: MemType) -> ProcessResult<Percent> {
		todo!()
	}

	pub fn chidren(&self) {
		todo!()
	}

	pub fn open_files(&self) -> ProcessResult<Vec<OpenFile>> {
		let mut open_files = Vec::new();

		for entry in fs::read_dir(self.procfs_path("fd"))
			.map_err(|e| io_error_to_process_error(e, self.pid))?
		{
			let path = entry
				.map_err(|e| io_error_to_process_error(e, self.pid))?
				.path();
			let fd = path
				.file_name()
				.unwrap()
				.to_string_lossy()
				.parse::<u32>()
				.unwrap();
			let open_file =
				fs::read_link(&path).map_err(|e| io_error_to_process_error(e, self.pid))?;

			open_files.push(OpenFile {
				fd: Some(fd),
				path: open_file,
			})
		}

		Ok(open_files)
	}

	pub fn connections(&self) {
		todo!()
	}

	pub fn connections_with_type(&self, _type: NetConnectionType) {
		todo!()
	}

	/// New method, not in Python psutil.
	pub fn replace(&mut self) -> bool {
		match Process::new(self.pid) {
			Ok(p) => {
				if p == *self {
					false
				} else {
					self.create_time = p.create_time;
					self.busy = p.busy;
					self.instant = p.instant;
					true
				}
			}
			Err(_) => false,
		}
	}

	/// Preemptively checks if the process is still alive.
	pub fn send_signal(&self, signal: Signal) -> ProcessResult<()> {
		if !self.is_running() {
			return Err(ProcessError::NoSuchProcess { pid: self.pid });
		}

		kill(unistd::Pid::from_raw(self.pid as i32), signal)
			.context(errors::NixError { pid: self.pid })
	}

	pub fn suspend(&self) {
		todo!()
	}

	pub fn resume(&self) {
		todo!()
	}

	pub fn terminate(&self) {
		todo!()
	}

	/// Preemptively checks if the process is still alive.
	pub fn kill(&self) -> ProcessResult<()> {
		self.send_signal(Signal::SIGKILL)
	}

	pub fn wait(&self) {
		todo!()
	}
}
