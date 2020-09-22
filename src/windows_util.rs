use std::mem::{align_of, size_of, zeroed};

use ntapi::ntrtl::RtlGetVersion;
use winapi::shared::minwindef::FILETIME;
use winapi::shared::ntdef::{LARGE_INTEGER, ULARGE_INTEGER, ULONG};
use winapi::shared::ntstatus::{
	STATUS_ACCESS_DENIED, STATUS_BUFFER_TOO_SMALL, STATUS_INFO_LENGTH_MISMATCH, STATUS_INVALID_CID,
};
use winapi::shared::winerror::{ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER};
use winapi::um::handleapi::{CloseHandle, DuplicateHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcess};
use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use winapi::um::winnt::{
	DUPLICATE_SAME_ACCESS, HANDLE, OSVERSIONINFOW, PROCESS_QUERY_INFORMATION,
	PROCESS_QUERY_LIMITED_INFORMATION, RTL_OSVERSIONINFOEXW, SYNCHRONIZE,
};
#[cfg(target_pointer_width = "32")]
use winapi::{
	shared::basetsd::{PULONG64, ULONG64},
	shared::minwindef::{BOOL, DWORD, PDWORD},
	shared::ntdef::{NTSTATUS, PVOID, PVOID64},
	um::libloaderapi::{GetModuleHandleA, GetProcAddress},
	um::processthreadsapi::GetCurrentProcessId,
	um::wow64apiset::IsWow64Process,
};

use crate::errors::{Error, WindowsOsError};
use crate::process::{ProcessError, ProcessResult};
use crate::Pid;
use crate::Result;
use ntapi::ntexapi::{
	NtQuerySystemInformation, SystemProcessInformation, SYSTEM_PROCESS_INFORMATION,
};
use winapi::ctypes::c_void;

pub(crate) fn windows_error_to_process_error(pid: Pid, e: WindowsOsError) -> ProcessError {
	match e {
		WindowsOsError::Win32Error { code, .. } => match code {
			ERROR_ACCESS_DENIED => ProcessError::AccessDenied { pid },
			// OpenProcess returns ERROR_INVALID_PARAMETER when process does not exist
			ERROR_INVALID_PARAMETER => ProcessError::NoSuchProcess { pid },
			_ => ProcessError::PsutilError {
				pid,
				source: Error::WindowsError { source: e },
			},
		},
		WindowsOsError::NtError { status, .. } => match status {
			STATUS_ACCESS_DENIED => ProcessError::AccessDenied { pid },
			// NtQuerySystemInformation returns STATUS_INVALID_CID when process does not exist
			STATUS_INVALID_CID => ProcessError::NoSuchProcess { pid },
			_ => ProcessError::PsutilError {
				pid,
				source: Error::WindowsError { source: e },
			},
		},
	}
}

#[derive(Debug)]
pub(crate) struct SafeHandle {
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
	#[inline]
	pub fn get_invalid() -> Self {
		Self {
			raw_handle: INVALID_HANDLE_VALUE,
		}
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
		if self.is_valid() {
			unsafe {
				let cp = GetCurrentProcess();
				let mut new_handle: HANDLE = zeroed();
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

				Self {
					raw_handle: new_handle,
				}
			}
		} else {
			Self {
				raw_handle: INVALID_HANDLE_VALUE,
			}
		}
	}
}

pub(crate) fn windows_filetime_to_ns(ft: &FILETIME) -> u64 {
	(((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64) * 100
}

pub(crate) unsafe fn large_integer_to_u64(i: &LARGE_INTEGER) -> u64 {
	let u = i.u();
	((u.HighPart as u64) << 32) | u.LowPart as u64
}

pub(crate) unsafe fn ularge_integer_to_u64(i: &ULARGE_INTEGER) -> u64 {
	let u = i.u();
	((u.HighPart as u64) << 32) | u.LowPart as u64
}

pub(crate) fn filetime_to_unix_time(ft: &FILETIME) -> u64 {
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
		RtlGetVersion(&mut vi as *mut _ as *mut OSVERSIONINFOW);

		(vi.dwMajorVersion, vi.dwMinorVersion)
	}
}

pub(crate) fn is_windows_version_higher_than_8_1() -> bool {
	let (major, minor) = get_windows_version();

	if major == 6 && minor >= 3 {
		true
	} else {
		major > 6
	}
}

#[inline(always)]
pub(crate) fn windows_filetime_default() -> FILETIME {
	FILETIME {
		dwLowDateTime: 0,
		dwHighDateTime: 0,
	}
}

#[inline(always)]
pub(crate) fn handle_invalid() -> HANDLE {
	INVALID_HANDLE_VALUE
}

#[inline(always)]
pub(crate) fn global_memory_status_ex() -> Result<MEMORYSTATUSEX> {
	unsafe {
		let mut msx: MEMORYSTATUSEX = zeroed();
		msx.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
		if GlobalMemoryStatusEx(&mut msx as *mut _) == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GlobalMemoryStatusEx",
			)));
		}

		Ok(msx)
	}
}

#[cfg(target_pointer_width = "32")]
pub(crate) fn get_current_process_handle() -> ProcessResult<SafeHandle> {
	unsafe {
		let pid = GetCurrentProcessId();
		let cp = GetCurrentProcess();
		let mut new_handle: HANDLE = zeroed();
		if DuplicateHandle(
			cp,
			cp,
			cp,
			&mut new_handle as *mut _,
			0,
			0,
			DUPLICATE_SAME_ACCESS,
		) == 0
		{
			return Err(windows_error_to_process_error(
				pid,
				WindowsOsError::last_win32_error("DuplicateHandle"),
			));
		}

		Ok(SafeHandle {
			raw_handle: new_handle,
		})
	}
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
pub(crate) struct Wow64Data {
	pub NtWow64QueryInformationProcess64: Option<
		unsafe extern "system" fn(
			ProcessHandle: HANDLE,
			ProcessInformationClass: DWORD,
			ProcessInformation: PVOID,
			ProcessInformationLength: DWORD,
			ReturnLength: PDWORD,
		) -> NTSTATUS,
	>,
	pub NtWow64ReadVirtualMemory64: Option<
		unsafe extern "system" fn(
			ProcessHandle: HANDLE,
			BaseAddress: PVOID64,
			Buffer: PVOID,
			Size: ULONG64,
			NumberOfBytesRead: PULONG64,
		) -> NTSTATUS,
	>,
	pub are_we_wow64: bool,
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
impl Wow64Data {
	pub fn init() -> Self {
		use std::mem::transmute;

		let mut are_we_wow64: BOOL = 0;
		if let Ok(current_process) = get_current_process_handle() {
			unsafe { IsWow64Process(current_process.get_raw(), &mut are_we_wow64 as *mut _) };
		}

		let ntdll = unsafe { GetModuleHandleA(b"ntdll.dll\x00".as_ptr() as *const _) };
		macro_rules! get_proc {
			($n:expr) => {{
				let t = unsafe { GetProcAddress(ntdll, $n.as_ptr() as *const _) };
				if t.is_null() {
					None
				} else {
					Some(unsafe { transmute(t) })
					}
				}};
		}

		if !ntdll.is_null() {
			let NtWow64QueryInformationProcess64: Option<
				unsafe extern "system" fn(
					ProcessHandle: HANDLE,
					ProcessInformationClass: DWORD,
					ProcessInformation: PVOID,
					ProcessInformationLength: DWORD,
					ReturnLength: PDWORD,
				) -> NTSTATUS,
			> = get_proc!(b"NtWow64QueryInformationProcess64\x00");

			let NtWow64ReadVirtualMemory64: Option<
				unsafe extern "system" fn(
					ProcessHandle: HANDLE,
					BaseAddress: PVOID64,
					Buffer: PVOID,
					Size: ULONG64,
					NumberOfBytesRead: PULONG64,
				) -> NTSTATUS,
			> = get_proc!(b"NtWow64ReadVirtualMemory64\x00");

			Wow64Data {
				NtWow64QueryInformationProcess64,
				NtWow64ReadVirtualMemory64,
				are_we_wow64: are_we_wow64 != 0,
			}
		} else {
			Wow64Data {
				NtWow64QueryInformationProcess64: None,
				NtWow64ReadVirtualMemory64: None,
				are_we_wow64: are_we_wow64 != 0,
			}
		}
	}
}

#[cfg(target_pointer_width = "32")]
#[ctor::ctor]
pub(crate) static WOW64: Wow64Data = Wow64Data::init();

pub(crate) struct ProcessInformation {
	pub(crate) pid: Pid,
	pub(crate) create_time_ns: u64,
	pub(crate) user_time_ns: u64,
	pub(crate) kernel_time_ns: u64,
}

impl ProcessInformation {
	pub(crate) fn get(pid: Pid) -> ProcessResult<Self> {
		for process in ProcessInformationArray::get()
			.map_err(|e| ProcessError::PsutilError { pid, source: e })?
		{
			if process.pid == pid {
				return Ok(process);
			}
		}
		Err(ProcessError::NoSuchProcess { pid })
	}
}

pub(crate) struct ProcessInformationArray {
	inner: Vec<SYSTEM_PROCESS_INFORMATION>,
	estimated_num_processes: u32,
}

impl ProcessInformationArray {
	pub(crate) fn get() -> Result<Self> {
		const INITIAL_SIZE: usize = 60;

		let mut buffer = Vec::<SYSTEM_PROCESS_INFORMATION>::with_capacity(INITIAL_SIZE);
		unsafe { buffer.set_len(INITIAL_SIZE) };

		let mut buffer_size_in_bytes: ULONG =
			(INITIAL_SIZE * size_of::<SYSTEM_PROCESS_INFORMATION>()) as ULONG;

		loop {
			let status = unsafe {
				NtQuerySystemInformation(
					SystemProcessInformation,
					buffer.as_mut_ptr() as *mut c_void,
					buffer_size_in_bytes,
					&mut buffer_size_in_bytes as *mut _,
				)
			};
			if status == 0 {
				let estimated_num_processes =
					buffer_size_in_bytes / size_of::<SYSTEM_PROCESS_INFORMATION>() as u32;

				return Ok(Self {
					inner: buffer,
					estimated_num_processes,
				});
			} else if status == STATUS_BUFFER_TOO_SMALL || status == STATUS_INFO_LENGTH_MISMATCH {
				let mut new_size_in_items =
					buffer_size_in_bytes as usize / size_of::<SYSTEM_PROCESS_INFORMATION>();
				if buffer_size_in_bytes as usize % size_of::<SYSTEM_PROCESS_INFORMATION>() != 0 {
					new_size_in_items += 1;
				}
				let additional = new_size_in_items - buffer.len();
				buffer.reserve(additional);
				unsafe { buffer.set_len(new_size_in_items) };
			} else {
				return Err(Error::from(WindowsOsError::nt_error(
					"NtQuerySystemInformation",
					status,
				)));
			}
		}
	}

	#[inline(always)]
	pub(crate) fn estimated_num_processes(&self) -> u32 {
		self.estimated_num_processes
	}
}

impl IntoIterator for ProcessInformationArray {
	type Item = ProcessInformation;
	type IntoIter = ProcessInformationIterator;

	fn into_iter(self) -> Self::IntoIter {
		ProcessInformationIterator {
			data: self,
			next: 0,
		}
	}
}

pub(crate) struct ProcessInformationIterator {
	data: ProcessInformationArray,
	next: u32,
}

impl Iterator for ProcessInformationIterator {
	type Item = ProcessInformation;

	fn next(&mut self) -> Option<Self::Item> {
		if self.data.inner.is_empty() || self.next == u32::MAX {
			return None;
		}

		let uptr = self.data.inner.as_ptr() as usize + self.next as usize;
		debug_assert_eq!(uptr % align_of::<SYSTEM_PROCESS_INFORMATION>(), 0);

		unsafe {
			let spi = uptr as *const SYSTEM_PROCESS_INFORMATION;
			let pid = (*spi).UniqueProcessId as usize as Pid;
			let create_time_ns = large_integer_to_u64(&(*spi).CreateTime) * 100;
			let user_time_ns = large_integer_to_u64(&(*spi).UserTime) * 100;
			let kernel_time_ns = large_integer_to_u64(&(*spi).KernelTime) * 100;

			if (*spi).NextEntryOffset == 0 {
				self.next = u32::MAX;
			} else {
				self.next += (*spi).NextEntryOffset;
			}

			/*let image_name_utf16 = ::std::slice::from_raw_parts(
				(*spi).ImageName.Buffer,
				(*spi).ImageName.Length as usize / 2,
			);
			let image_name_utf8 = String::from_utf16(image_name_utf16).unwrap();
			println!("{} {}", pid, image_name_utf8);*/

			Some(ProcessInformation {
				pid,
				create_time_ns,
				user_time_ns,
				kernel_time_ns,
			})
		}
	}
}
