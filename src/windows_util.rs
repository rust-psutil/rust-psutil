use crate::errors::{Error, WindowsOsError};
use crate::process::ProcessError;
use crate::Pid;
use std::mem::{size_of, transmute, zeroed, MaybeUninit};

use winapi::shared::minwindef::FILETIME;
use winapi::shared::ntdef::{LARGE_INTEGER, ULARGE_INTEGER};
use winapi::shared::ntstatus;
use winapi::shared::winerror::ERROR_ACCESS_DENIED;
use winapi::um::handleapi::{CloseHandle, DuplicateHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcess};
use winapi::um::winnt::{
	DUPLICATE_SAME_ACCESS, HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
	RTL_OSVERSIONINFOEXW, SYNCHRONIZE,
};

use ntapi::ntrtl::RtlGetVersion;

pub(crate) fn windows_error_to_process_error(pid: Pid, e: WindowsOsError) -> ProcessError {
	match e {
		WindowsOsError::Win32Error { code, .. } => match code {
			ERROR_ACCESS_DENIED => ProcessError::AccessDenied { pid },
			// TODO: handle other cases
			_ => ProcessError::PsutilError {
				pid,
				source: Error::WindowsError { source: e },
			},
		},
		WindowsOsError::NtError { status, .. } => match status {
			ntstatus::STATUS_ACCESS_DENIED => ProcessError::AccessDenied { pid },
			_ => ProcessError::PsutilError {
				pid,
				source: Error::WindowsError { source: e },
			},
		},
	}
}

#[derive(Debug)]
pub struct SafeHandle {
	raw_handle: HANDLE,
}

impl SafeHandle {
	#[inline]
	pub fn from_raw_handle(raw_handle: HANDLE) -> Self {
		Self { raw_handle }
	}
	#[inline]
	pub fn get_raw(&self) -> HANDLE {
		self.raw_handle
	}
	#[inline]
	pub fn is_valid(&self) -> bool {
		self.raw_handle != INVALID_HANDLE_VALUE
	}
}

impl Drop for SafeHandle {
	fn drop(&mut self) {
		if self.is_valid() {
			unsafe { CloseHandle(self.raw_handle) };
		}
	}
}

impl Clone for SafeHandle {
	fn clone(&self) -> Self {
		unsafe {
			let cp = GetCurrentProcess();
			let mut new_handle: HANDLE = MaybeUninit::uninit().assume_init();
			if DuplicateHandle(
				cp,
				self.get_raw(),
				cp,
				&mut new_handle as *mut _,
				0,
				0,
				DUPLICATE_SAME_ACCESS,
			) == 0
			{
				new_handle = INVALID_HANDLE_VALUE;
			}

			return Self {
				raw_handle: new_handle,
			};
		}
	}
}

pub(crate) fn windows_filetime_to_ns(ft: &FILETIME) -> u64 {
	(((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64) * 100
}

pub unsafe fn large_integer_to_u64(i: &LARGE_INTEGER) -> u64 {
	let u = i.u();
	((u.HighPart as u64) << 32) | u.LowPart as u64
}

pub unsafe fn ularge_integer_to_u64(i: &ULARGE_INTEGER) -> u64 {
	let u = i.u();
	((u.HighPart as u64) << 32) | u.LowPart as u64
}

pub fn filetime_to_unix_time(ft: &FILETIME) -> u64 {
	((((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64) - 116444736000000000u64)
		/ 10000000u64
}

pub(crate) unsafe fn try_open_process_for_limited_query(
	pid: Pid,
) -> ::std::result::Result<(SafeHandle, u32), WindowsOsError> {
	const NULL_HANDLE: HANDLE = 0isize as HANDLE;

	// SYNCHRONIZE required for sys_wait
	let access_rights = PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE;

	match OpenProcess(access_rights, false as i32, pid) {
		NULL_HANDLE => Err(WindowsOsError::last_win32_error("OpenProcess")),
		handle => Ok((SafeHandle::from_raw_handle(handle), access_rights)),
	}
}

pub(crate) unsafe fn try_open_process_for_query(
	pid: Pid,
	can_be_limited: bool,
) -> ::std::result::Result<(SafeHandle, u32), WindowsOsError> {
	if can_be_limited {
		if let Ok((handle, access_rights)) = try_open_process_for_limited_query(pid) {
			return Ok((handle, access_rights));
		}
	}

	const NULL_HANDLE: HANDLE = 0isize as HANDLE;

	// SYNCHRONIZE required for sys_wait
	let access_rights = PROCESS_QUERY_INFORMATION | SYNCHRONIZE;

	match OpenProcess(access_rights, false as i32, pid) {
		NULL_HANDLE => Err(WindowsOsError::last_win32_error("OpenProcess")),
		handle => Ok((SafeHandle::from_raw_handle(handle), access_rights)),
	}
}

pub(crate) fn get_windows_version() -> (u32, u32) {
	unsafe {
		let mut vi: RTL_OSVERSIONINFOEXW = zeroed();
		vi.dwOSVersionInfoSize = size_of::<RTL_OSVERSIONINFOEXW>() as u32;
		RtlGetVersion(transmute(&mut vi as *mut _));

		(vi.dwMajorVersion, vi.dwMinorVersion)
	}
}

pub(crate) fn is_windows_version_higher_than_8_1() -> bool {
	let (major, minor) = get_windows_version();

	if major == 6 && minor >= 3 {
		true
	} else if major > 6 {
		true
	} else {
		false
	}
}
