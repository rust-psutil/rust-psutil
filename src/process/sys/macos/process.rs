// https://github.com/heim-rs/heim/blob/master/heim-process/src/sys/macos/process/mod.rs
// https://github.com/heim-rs/heim/blob/master/heim-process/src/sys/macos/utils.rs

use std::convert::{From, TryFrom};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::common::NetConnectionType;
use crate::process::os::macos::{kinfo_proc, kinfo_process, kinfo_processes};
use crate::process::{
	io_error_to_process_error, MemType, OpenFile, Process, ProcessCpuTimes, ProcessError,
	ProcessResult, Status,
};
use crate::{Count, Percent, Pid};

// fn catch_zombie<T: Into<ProcessError>>(e: T, pid: Pid) -> ProcessError {
// 	match e.into() {
// 		ProcessError::Load(ref e) if e.raw_os_error() == Some(libc::ESRCH) => {
// 			let kinfo_proc = match bindings::process(pid) {
// 				Ok(info) => info,
// 				Err(e) => return e,
// 			};

// 			match Status::try_from(kinfo_proc.kp_proc.p_stat) {
// 				Ok(Status::Zombie) => ProcessError::ZombieProcess(pid),
// 				Ok(_) => ProcessError::AccessDenied(pid),
// 				Err(e) => e.into(),
// 			}
// 		}
// 		other => other,
// 	}
// }

impl From<kinfo_proc> for Process {
	fn from(kinfo_proc: kinfo_proc) -> Process {
		let timeval = unsafe {
			// TODO: How can it be guaranteed that in this case
			// `p_un.p_starttime` will be filled correctly?
			kinfo_proc.kp_proc.p_un.p_starttime
		};
		let create_time = Duration::from_secs(timeval.tv_sec as u64)
			+ Duration::from_micros(timeval.tv_usec as u64);

		Process {
			pid: kinfo_proc.kp_proc.p_pid,
			create_time,
		}
	}
}

impl Process {
	pub(crate) fn sys_new(pid: Pid) -> ProcessResult<Process> {
		match kinfo_process(pid) {
			Ok(kinfo_proc) => Ok(kinfo_proc.into()),
			Err(e) => Err(e),
			// Err(e) => catch_zombie(io_error_to_process_error(e, pid), pid),
		}
	}

	pub(crate) fn sys_ppid(&self) -> ProcessResult<Option<Pid>> {
		todo!()
	}

	pub(crate) fn sys_name(&self) -> ProcessResult<String> {
		todo!()
	}

	pub(crate) fn sys_exe(&self) -> ProcessResult<PathBuf> {
		todo!()
	}

	pub(crate) fn sys_cmdline(&self) -> ProcessResult<Option<String>> {
		todo!()
	}

	pub(crate) fn sys_cmdline_vec(&self) -> ProcessResult<Option<Vec<String>>> {
		todo!()
	}

	pub(crate) fn sys_parents(&self) -> Option<Vec<Process>> {
		todo!()
	}

	pub(crate) fn sys_status(&self) -> ProcessResult<Status> {
		todo!()
	}

	pub(crate) fn sys_cwd(&self) -> ProcessResult<PathBuf> {
		todo!()
	}

	pub(crate) fn sys_username(&self) -> String {
		todo!()
	}

	pub(crate) fn sys_get_nice(&self) -> i32 {
		todo!()
	}

	pub(crate) fn sys_set_nice(&self, _nice: i32) {
		todo!()
	}

	pub(crate) fn sys_num_ctx_switches(&self) -> Count {
		todo!()
	}

	pub(crate) fn sys_num_threads(&self) -> Count {
		todo!()
	}

	pub(crate) fn sys_threads(&self) {
		todo!()
	}

	pub(crate) fn sys_cpu_times(&self) -> ProcessResult<ProcessCpuTimes> {
		todo!()
	}

	pub(crate) fn sys_cpu_percent(&mut self) -> ProcessResult<Percent> {
		todo!()
	}

	pub(crate) fn sys_memory_info(&self) {
		todo!()
	}

	pub(crate) fn sys_memory_full_info(&self) {
		todo!()
	}

	pub(crate) fn sys_memory_percent(&self) -> ProcessResult<Percent> {
		todo!()
	}

	pub(crate) fn sys_memory_percent_with_type(&self, _type: MemType) -> ProcessResult<Percent> {
		todo!()
	}

	pub(crate) fn sys_chidren(&self) {
		todo!()
	}

	pub(crate) fn sys_open_files(&self) -> ProcessResult<Vec<OpenFile>> {
		todo!()
	}

	pub(crate) fn sys_connections(&self) {
		todo!()
	}

	pub(crate) fn sys_connections_with_type(&self, _type: NetConnectionType) {
		todo!()
	}

	pub(crate) fn sys_wait(&self) {
		todo!()
	}
}

pub fn processes() -> io::Result<Vec<ProcessResult<Process>>> {
	Ok(kinfo_processes()?
		.into_iter()
		.map(|proc| Ok(proc.into()))
		.collect())
}
