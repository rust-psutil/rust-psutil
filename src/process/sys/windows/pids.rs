use winapi::shared::ntstatus::STATUS_ACCESS_DENIED;
use winapi::shared::winerror::ERROR_ACCESS_DENIED;

use crate::windows_util::{try_open_process_for_query, ProcessInformationArray};
use crate::{Pid, Result, WindowsOsError};

pub fn pids() -> Result<Vec<Pid>> {
	let processes = ProcessInformationArray::get()?;
	let mut pids: Vec<Pid> = Vec::with_capacity(processes.estimated_num_processes() as usize);

	for process in processes {
		pids.push(process.pid);
	}

	Ok(pids)
}

pub fn pid_exists(pid: Pid) -> bool {
	match unsafe { try_open_process_for_query(pid, true) } {
		Ok(_)
		| Err(WindowsOsError::Win32Error {
			code: ERROR_ACCESS_DENIED,
			..
		}) => true,
		Err(WindowsOsError::NtError {
			status: STATUS_ACCESS_DENIED,
			..
		}) => true,
		Err(_) => false,
	}
}
