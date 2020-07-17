use crate::common::NetConnectionType;
use crate::process::{
	MemType, MemoryInfo, OpenFile, Process, ProcessCpuTimes, ProcessError, ProcessResult, Status,
};
use crate::windows_util::*;
use crate::{Count, Error, Percent, Pid, Result, WindowsOsError};
use std::cmp::min;
use std::ffi::c_void;
use std::mem;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::ptr;
use std::thread::sleep;
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{FILETIME, MAX_PATH};
use winapi::shared::ntdef::UNICODE_STRING;
use winapi::shared::ntstatus::{
	STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INFO_LENGTH_MISMATCH, STATUS_NOT_FOUND,
	STATUS_SUCCESS,
};
use winapi::shared::winerror::{ERROR_GEN_FAILURE, ERROR_INVALID_HANDLE, ERROR_PARTIAL_COPY};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::DuplicateHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::processthreadsapi::{GetCurrentProcess, GetProcessTimes, TerminateProcess};
use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use winapi::um::shellapi::CommandLineToArgvW;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::tlhelp32::{
	CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use winapi::um::winbase::{LocalFree, QueryFullProcessImageNameW, INFINITE};
use winapi::um::winnt::{
	HANDLE, MEMORY_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_SUSPEND_RESUME,
	PROCESS_TERMINATE, PROCESS_VM_READ,
};

use ntapi::ntpsapi::{
	NtQueryInformationProcess, NtResumeProcess, NtSuspendProcess, ProcessBasicInformation,
	ProcessCommandLineInformation, ProcessWow64Information, PROCESS_BASIC_INFORMATION,
};

use super::ntapi::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ProcessDataKind {
	CMDLINE,
	CWD,
	ENVIRONMENT,
}

impl Process {
	fn get_handle(&self) -> ProcessResult<&SafeHandle> {
		if self.handle.is_valid() {
			Ok(&self.handle)
		} else {
			// this could happen if DuplicateHandle() in SafeHandle clone() fails
			Err(ProcessError::PsutilError {
				pid: self.pid,
				source: Error::from(WindowsOsError::from_code(
					ERROR_INVALID_HANDLE,
					"Process::get_handle()",
				)),
			})
		}
	}

	fn raise_privileges(&self, more_rights: u32) -> ProcessResult<SafeHandle> {
		unsafe {
			let current_process = GetCurrentProcess();
			let mut handle: HANDLE = handle_invalid();
			if DuplicateHandle(
				current_process,
				self.get_handle()?.get_raw(),
				current_process,
				&mut handle as *mut _,
				self.access_rights | more_rights,
				0,
				0,
			) == 0
			{
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("DuplicateHandle"),
				));
			}

			Ok(SafeHandle::from_raw_handle(handle))
		}
	}

	pub(crate) fn sys_new(pid: Pid) -> ProcessResult<Process> {
		get_process_info(pid)
	}

	pub(crate) fn sys_ppid(&self) -> ProcessResult<Option<Pid>> {
		todo!()
	}

	pub(crate) fn sys_name(&self) -> ProcessResult<String> {
		match self.sys_exe()?.file_name() {
			Some(s) => match s.to_str() {
				Some(s) => Ok(s.to_owned()),
				None => Ok("<unknown>".to_owned()),
			},
			None => Ok("<unknown>".to_owned()),
		}
	}

	pub(crate) fn sys_exe(&self) -> ProcessResult<PathBuf> {
		unsafe {
			let mut buffer: [MaybeUninit<u16>; MAX_PATH] = MaybeUninit::uninit().assume_init();
			let mut size: u32 = MAX_PATH as u32;
			let handle = self.get_handle()?;
			if QueryFullProcessImageNameW(
				handle.get_raw(),
				0,
				buffer.as_mut_ptr() as *mut u16,
				&mut size as *mut _,
			) == 0
			{
				let code = GetLastError();
				if code == ERROR_GEN_FAILURE
					&& QueryFullProcessImageNameW(
						handle.get_raw(),
						1, /* PROCESS_NAME_NATIVE */
						buffer.as_mut_ptr() as *mut u16,
						&mut size as *mut _,
					) != 0
				{
					// Attempt to query win32 path of WSL processes returns ERROR_GEN_FAILURE
					// use NT path

					// TODO: We could try convert that to windows path manually
					// or using undocumented NT api
					// RtlNtPathNameToDosPathName
				} else {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::Win32Error {
							call: "QueryFullProcessImageNameW",
							code,
						},
					));
				}
			}

			match String::from_utf16(
				&*(&buffer[..size as usize] as *const [std::mem::MaybeUninit<u16>]
					as *const [u16]),
			) {
				Ok(d) => Ok(PathBuf::from(d)),
				Err(e) => Err(ProcessError::PsutilError {
					pid: self.pid,
					source: Error::from(e),
				}),
			}
		}
	}

	pub(crate) fn sys_cmdline(&self) -> ProcessResult<Option<String>> {
		Ok(self.cmdline_vec()?.map(|c| c.join(" ")))
	}

	pub(crate) fn sys_cmdline_vec(&self) -> ProcessResult<Option<Vec<String>>> {
		let c = unsafe { self.get_command_line()? };
		let mut num_args: u32 = 0;
		unsafe {
			let p = CommandLineToArgvW(c.as_ptr(), &mut num_args as *mut _ as *mut i32);
			if p.is_null() {
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("CommandLineToArgvW"),
				));
			}

			let mut cmdline: Vec<String> = Vec::with_capacity(num_args as usize);

			for i in 0..num_args {
				let q = ptr::read(p.offset(i as isize));
				let length = Self::string_length(q);
				let slice = ::std::slice::from_raw_parts::<u16>(q, length);

				match String::from_utf16(slice) {
					Ok(s) => cmdline.push(s),
					Err(e) => {
						return Err(ProcessError::PsutilError {
							pid: self.pid,
							source: Error::from(e),
						})
					}
				};
			}

			LocalFree(p as *mut c_void);

			Ok(Some(cmdline))
		}
	}

	pub(crate) fn sys_parents(&self) -> Option<Vec<Process>> {
		todo!()
	}

	pub(crate) fn sys_status(&self) -> ProcessResult<Status> {
		todo!()
	}

	pub(crate) fn sys_cwd(&self) -> ProcessResult<PathBuf> {
		let mut delay = 1;
		for _ in 0..33 {
			match unsafe { self.get_process_data(ProcessDataKind::CWD) } {
				Err(ProcessError::PsutilError {
					source:
						Error::WindowsError {
							source:
								WindowsOsError::Win32Error {
									code: ERROR_PARTIAL_COPY,
									..
								},
						},
					..
				}) => {
					sleep(Duration::from_millis(delay));
					delay = min(delay * 2, 40);
				}
				Err(e) => return Err(e),
				Ok(x) => {
					match String::from_utf16(&x) {
						Ok(x) => return Ok(PathBuf::from(x)),
						Err(e) => {
							return Err(ProcessError::PsutilError {
								pid: self.pid,
								source: Error::FromUtf16ConvertError { source: e },
							})
						}
					};
				}
			};
		}

		Err(ProcessError::AccessDenied { pid: self.pid })
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
		unsafe {
			let mut creation_time: FILETIME = windows_filetime_default();
			let mut exit_time: FILETIME = windows_filetime_default();
			let mut kernel_time: FILETIME = windows_filetime_default();
			let mut user_time: FILETIME = windows_filetime_default();

			if GetProcessTimes(
				self.get_handle()?.get_raw(),
				&mut creation_time as *mut _,
				&mut exit_time as *mut _,
				&mut kernel_time as *mut _,
				&mut user_time as *mut _,
			) != 0
			{
				let user_time_nanos = windows_filetime_to_ns(&user_time);
				let kernel_time_nanos = windows_filetime_to_ns(&kernel_time);

				Ok(ProcessCpuTimes {
					user: Duration::from_nanos(user_time_nanos),
					system: Duration::from_nanos(kernel_time_nanos),
					// TODO:
					children_user: Duration::from_nanos(0),
					children_system: Duration::from_nanos(0),
				})
			} else {
				Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("GetProcessTimes"),
				))
			}
		}
	}

	pub(crate) fn sys_memory_info(&self) -> ProcessResult<MemoryInfo> {
		unsafe {
			let mut pmc: PROCESS_MEMORY_COUNTERS = mem::zeroed();
			if GetProcessMemoryInfo(
				self.get_handle()?.get_raw(),
				&mut pmc as *mut _,
				mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
			) == 0
			{
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("GetProcessMemoryInfo"),
				));
			}

			Ok(MemoryInfo {
				rss: pmc.WorkingSetSize as u64,
				vms: pmc.PagefileUsage as u64,
			})
		}
	}

	pub(crate) fn sys_memory_full_info(&self) {
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
		if let Ok(handle) = self.get_handle() {
			unsafe {
				WaitForSingleObject(handle.get_raw(), INFINITE);
			}
		}
	}

	pub(crate) fn terminate_process(&self) -> ProcessResult<()> {
		let handle = self.raise_privileges(PROCESS_TERMINATE)?;
		match unsafe { TerminateProcess(handle.get_raw(), -1i32 as u32) } {
			0 => Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::last_win32_error("TerminateProcess"),
			)),
			_ => Ok(()),
		}
	}

	pub(crate) fn suspend_process(&self) -> ProcessResult<()> {
		let handle = self.raise_privileges(PROCESS_SUSPEND_RESUME)?;

		match unsafe { NtSuspendProcess(handle.get_raw()) } {
			0 => Ok(()),
			e => Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::nt_error("NtSuspendProcess", e),
			)),
		}
	}
	pub(crate) fn resume_process(&self) -> ProcessResult<()> {
		let handle = self.raise_privileges(PROCESS_SUSPEND_RESUME)?;

		match unsafe { NtResumeProcess(handle.get_raw()) } {
			0 => Ok(()),
			e => Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::nt_error("NtResumeProcess", e),
			)),
		}
	}

	#[allow(non_snake_case)]
	pub(crate) unsafe fn get_process_data(
		&self,
		data_kind: ProcessDataKind,
	) -> ProcessResult<Vec<u16>> {
		let handle = self.raise_privileges(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)?;
		let mut ppeb32: *mut u8 = ptr::null_mut();

		let mut src: *const u8 = ptr::null();
		let mut size: usize = 0;

		#[cfg(target_pointer_width = "64")]
		{
			match NtQueryInformationProcess(
				handle.get_raw(),
				ProcessWow64Information,
				&mut ppeb32 as *mut _ as *mut c_void,
				mem::size_of::<*const u8>() as u32,
				ptr::null_mut(),
			) {
				0 => (),
				e => {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtQueryInformationProcess", e),
					));
				}
			}

			if !ppeb32.is_null() {
				let mut peb32: PEB32 = mem::zeroed();
				let mut procParameters32: RTL_USER_PROCESS_PARAMETERS32 = mem::zeroed();

				if ReadProcessMemory(
					handle.get_raw(),
					ppeb32 as *const c_void,
					&mut peb32 as *mut _ as *mut c_void,
					mem::size_of::<PEB32>(),
					ptr::null_mut(),
				) == 0
				{
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::last_win32_error("ReadProcessMemory"),
					));
				}

				if ReadProcessMemory(
					handle.get_raw(),
					mem::transmute(peb32.ProcessParameters as u64),
					&mut procParameters32 as *mut _ as *mut c_void,
					mem::size_of::<RTL_USER_PROCESS_PARAMETERS32>(),
					ptr::null_mut(),
				) == 0
				{
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::last_win32_error("ReadProcessMemory"),
					));
				}

				match data_kind {
					ProcessDataKind::CMDLINE => {
						src = mem::transmute(procParameters32.CommandLine.Buffer as u64);
						size = procParameters32.CommandLine.Length as usize;
					}
					ProcessDataKind::CWD => {
						src = mem::transmute(procParameters32.CurrentDirectoryPath.Buffer as u64);
						size = procParameters32.CurrentDirectoryPath.Length as usize;
					}
					ProcessDataKind::ENVIRONMENT => {
						src = mem::transmute(procParameters32.env as u64);
					}
				};
			}
		}
		#[cfg(target_pointer_width = "32")]
		{
			todo!();
		}

		#[cfg(target_pointer_width = "64")]
		type UintPtr = u64;
		#[cfg(target_pointer_width = "32")]
		type UintPtr = u32;

		if src.is_null() {
			let mut pbi: PROCESS_BASIC_INFORMATION = mem::zeroed();
			let mut peb: PEB_ = mem::zeroed();
			let mut procParameters: RTL_USER_PROCESS_PARAMETERS_ = mem::zeroed();

			match NtQueryInformationProcess(
				handle.get_raw(),
				ProcessBasicInformation,
				&mut pbi as *mut _ as *mut c_void,
				mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
				ptr::null_mut(),
			) {
				0 => (),
				e => {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtQueryInformationProcess", e),
					));
				}
			}

			if ReadProcessMemory(
				handle.get_raw(),
				mem::transmute(pbi.PebBaseAddress as UintPtr),
				&mut peb as *mut _ as *mut c_void,
				mem::size_of::<PEB_>(),
				ptr::null_mut(),
			) == 0
			{
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("ReadProcessMemory"),
				));
			}

			if ReadProcessMemory(
				handle.get_raw(),
				mem::transmute(peb.ProcessParameters as UintPtr),
				&mut procParameters as *mut _ as *mut c_void,
				mem::size_of::<RTL_USER_PROCESS_PARAMETERS_>(),
				ptr::null_mut(),
			) == 0
			{
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("ReadProcessMemory"),
				));
			}

			match data_kind {
				ProcessDataKind::CMDLINE => {
					src = mem::transmute(procParameters.CommandLine.Buffer as UintPtr);
					size = procParameters.CommandLine.Length as usize;
				}
				ProcessDataKind::CWD => {
					src = mem::transmute(procParameters.CurrentDirectoryPath.Buffer as UintPtr);
					size = procParameters.CurrentDirectoryPath.Length as usize;
				}
				ProcessDataKind::ENVIRONMENT => {
					src = mem::transmute(procParameters.env as UintPtr);
				}
			}
		}

		if data_kind == ProcessDataKind::ENVIRONMENT {
			#[cfg(target_pointer_width = "32")]
			{
				/*
				if (weAreWow64 && !theyAreWow64) {
					AccessDenied("can't query 64-bit process in 32-bit-WoW mode");
					goto error;
				}
				else*/
				todo!();
			}
			size = self.get_process_region_size(&handle, mem::transmute(src))?;
		}

		let mut buffer: Vec<u16> = Vec::with_capacity(size / 2 + 1);
		buffer.set_len(buffer.capacity());

		#[cfg(target_pointer_width = "32")]
		{
			/*if (weAreWow64 && !theyAreWow64) {
				status = NtWow64ReadVirtualMemory64(
					hProcess,
					src64,
					buffer,
					size,
					NULL);
				if (!NT_SUCCESS(status)) {
					psutil_SetFromNTStatusErr(status, "NtWow64ReadVirtualMemory64");
					goto error;
				}
			} else*/
			todo!();
		}
		if ReadProcessMemory(
			handle.get_raw(),
			src as *const c_void,
			buffer.as_mut_ptr() as *mut c_void,
			size,
			ptr::null_mut(),
		) == 0
		{
			return Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::last_win32_error("ReadProcessMemory"),
			));
		}

		let buffer_len = buffer.len();
		buffer[buffer_len - 1] = 0;
		Ok(buffer)
	}

	unsafe fn get_process_region_size(
		&self,
		handle: &SafeHandle,
		src: *const u8,
	) -> ProcessResult<usize> {
		#[cfg(target_pointer_width = "64")]
		#[allow(dead_code)]
		type UintPtr = u64;
		#[cfg(target_pointer_width = "32")]
		#[allow(dead_code)]
		type UintPtr = u32;

		let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();

		if VirtualQueryEx(
			handle.get_raw(),
			src as *const c_void,
			&mut mbi as *mut _,
			mem::size_of::<MEMORY_BASIC_INFORMATION>(),
		) == 0
		{
			return Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::last_win32_error("VirtualQueryEx"),
			));
		}

		let region_size = mbi.RegionSize as UintPtr;
		let src = mem::transmute::<*const u8, UintPtr>(src);
		let base_address = mem::transmute::<*mut ::std::ffi::c_void, UintPtr>(mbi.BaseAddress);

		let size = region_size - (src - base_address);

		Ok(size as usize)
	}

	unsafe fn string_length(p: *const u16) -> usize {
		let mut len: usize = 0;
		let mut p = p;
		loop {
			if ptr::read(p) == 0 {
				break;
			}
			len += 1;
			p = p.offset(1);
		}

		len
	}
	unsafe fn get_command_line_using_peb(&self) -> ProcessResult<Vec<u16>> {
		let mut delay = 1;
		for _ in 0..33 {
			match self.get_process_data(ProcessDataKind::CMDLINE) {
				Err(ProcessError::PsutilError {
					source:
						Error::WindowsError {
							source:
								WindowsOsError::Win32Error {
									code: ERROR_PARTIAL_COPY,
									..
								},
						},
					..
				}) => {
					sleep(Duration::from_millis(delay));
					delay = min(delay * 2, 40);
				}
				Err(e) => return Err(e),
				Ok(r) => return Ok(r),
			};
		}

		Err(ProcessError::AccessDenied { pid: self.pid })
	}
	unsafe fn get_command_line(&self) -> ProcessResult<Vec<u16>> {
		if let Ok(r) = self.get_command_line_using_peb() {
			return Ok(r);
		}

		if !is_windows_version_higher_than_8_1() {
			// not supported on pre win 8.1
			return Err(ProcessError::AccessDenied { pid: self.pid });
		}

		let handle = self.get_handle()?;

		let mut buffer_len: u32 = 0;

		let mut status = NtQueryInformationProcess(
			handle.get_raw(),
			ProcessCommandLineInformation,
			ptr::null_mut(),
			0,
			&mut buffer_len as *mut _,
		);

		if status == STATUS_NOT_FOUND {
			return Err(ProcessError::AccessDenied { pid: self.pid });
		}

		if status != STATUS_BUFFER_OVERFLOW
			&& status != STATUS_BUFFER_TOO_SMALL
			&& status != STATUS_INFO_LENGTH_MISMATCH
		{
			return Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::nt_error("NtQueryInformationProcess", status),
			));
		}

		let mut buffer: Vec<u8> = Vec::with_capacity(buffer_len as usize);

		status = NtQueryInformationProcess(
			handle.get_raw(),
			ProcessCommandLineInformation,
			buffer.as_mut_ptr() as *mut c_void,
			buffer_len,
			&mut buffer_len as *mut _,
		);
		if status != STATUS_SUCCESS {
			return Err(windows_error_to_process_error(
				self.pid,
				WindowsOsError::nt_error("NtQueryInformationProcess", status),
			));
		}

		// copy to force proper alignment
		let mut us: UNICODE_STRING = mem::zeroed();
		ptr::copy::<u8>(
			buffer.as_ptr(),
			&mut us as *mut _ as *mut u8,
			mem::size_of::<UNICODE_STRING>(),
		);
		let len = Self::string_length(us.Buffer);

		Ok(::std::slice::from_raw_parts::<u16>(us.Buffer, len)
			.iter()
			.copied()
			.chain(::std::iter::once(0))
			.collect::<Vec<u16>>())
	}
}

fn get_process_info(pid: Pid) -> ProcessResult<Process> {
	let create_time: Duration;
	let busy: Duration;

	match unsafe { try_open_process_for_query(pid, true) } {
		Ok((handle, access_rights)) => unsafe {
			let mut creation_time: FILETIME = windows_filetime_default();
			let mut exit_time: FILETIME = windows_filetime_default();
			let mut kernel_time: FILETIME = windows_filetime_default();
			let mut user_time: FILETIME = windows_filetime_default();

			if GetProcessTimes(
				handle.get_raw(),
				&mut creation_time as *mut _,
				&mut exit_time as *mut _,
				&mut kernel_time as *mut _,
				&mut user_time as *mut _,
			) != 0
			{
				let create_time_raw = windows_filetime_to_ns(&creation_time);
				let user_time_raw = windows_filetime_to_ns(&user_time);
				let kernel_time_raw = windows_filetime_to_ns(&kernel_time);

				create_time = Duration::from_nanos(create_time_raw);
				busy = Duration::from_nanos(user_time_raw + kernel_time_raw);
			} else {
				create_time = Duration::from_nanos(0);
				busy = Duration::from_nanos(0);
			}

			Ok(Process {
				pid,
				create_time,
				busy,
				instant: Instant::now(),
				handle,
				access_rights,
			})
		},
		Err(error) => Err(windows_error_to_process_error(pid, error)),
	}
}

fn process_entry(pe: &PROCESSENTRY32W, pl: &mut Vec<ProcessResult<Process>>) {
	pl.push(get_process_info(pe.th32ProcessID));
}

pub fn processes() -> Result<Vec<ProcessResult<Process>>> {
	let mut pe: PROCESSENTRY32W = unsafe { mem::zeroed() };
	pe.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

	let mut pl: Vec<ProcessResult<Process>> = Vec::new();

	unsafe {
		let snapshot = SafeHandle::from_raw_handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0));
		if !snapshot.is_valid() {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"CreateToolhelp32Snapshot",
			)));
		}

		if Process32FirstW(snapshot.get_raw(), &mut pe as *mut _) == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"CreateToolhelp32Snapshot",
			)));
		}

		process_entry(&pe, &mut pl);

		while Process32NextW(snapshot.get_raw(), &mut pe as *mut _) != 0 {
			process_entry(&pe, &mut pl);
		}
	}

	Ok(pl)
}
