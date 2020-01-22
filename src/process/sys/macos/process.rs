use std::path::PathBuf;

use crate::common::NetConnectionType;
use crate::process::{MemType, OpenFile, Process, ProcessCpuTimes, ProcessResult, Status};
use crate::{Count, Percent, Pid};

impl Process {
	pub(crate) fn sys_new(pid: Pid) -> ProcessResult<Process> {
		todo!()
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

	pub(crate) fn sys_parent(&self) -> ProcessResult<Option<Process>> {
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
