use crate::windows_util::{try_open_process_for_query, SafeHandle};
use crate::{Error, Pid, Result, WindowsOsError};
use std::mem;

use winapi::shared::ntstatus::STATUS_ACCESS_DENIED;
use winapi::shared::winerror::ERROR_ACCESS_DENIED;

use winapi::um::tlhelp32::{
	CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, LPPROCESSENTRY32W, PROCESSENTRY32W,
	TH32CS_SNAPPROCESS,
};

pub fn pids() -> Result<Vec<Pid>> {
	unsafe {
		let mut pe: PROCESSENTRY32W = mem::zeroed();
		pe.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

		let snapshot = SafeHandle::from_raw_handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0));
		if !snapshot.is_valid() {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"CreateToolhelp32Snapshot",
			)));
		}

		if Process32FirstW(snapshot.get_raw(), &mut pe as LPPROCESSENTRY32W) == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"CreateToolhelp32Snapshot",
			)));
		}

		let mut pids: Vec<Pid> = Vec::with_capacity(256);
		pids.push(pe.th32ProcessID);

		while Process32NextW(snapshot.get_raw(), &mut pe as LPPROCESSENTRY32W) != 0 {
			pids.push(pe.th32ProcessID);
		}

		Ok(pids)
	}
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
