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

#[cfg(target_pointer_width = "64")]
use ntapi::ntpsapi::ProcessWow64Information;
use ntapi::ntpsapi::{
	NtQueryInformationProcess, NtResumeProcess, NtSuspendProcess, ProcessBasicInformation,
	ProcessCommandLineInformation, PROCESS_BASIC_INFORMATION,
};
use winapi::shared::minwindef::{FILETIME, MAX_PATH};
use winapi::shared::ntdef::{UNICODE_STRING, USHORT};
use winapi::shared::ntstatus::{
	STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_INFO_LENGTH_MISMATCH, STATUS_NOT_FOUND,
	STATUS_SUCCESS,
};
use winapi::shared::winerror::{
	ERROR_ACCESS_DENIED, ERROR_GEN_FAILURE, ERROR_INVALID_PARAMETER, ERROR_PARTIAL_COPY,
	WAIT_TIMEOUT,
};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::DuplicateHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::processthreadsapi::{GetCurrentProcess, GetProcessTimes, TerminateProcess};
use winapi::um::psapi::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use winapi::um::shellapi::CommandLineToArgvW;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::{LocalFree, QueryFullProcessImageNameW, INFINITE};
use winapi::um::winnt::{
	HANDLE, MEMORY_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_SUSPEND_RESUME,
	PROCESS_TERMINATE, PROCESS_VM_READ,
};
#[cfg(target_pointer_width = "32")]
use winapi::um::wow64apiset::IsWow64Process;

use super::ntapi::*;
use ntapi::ntexapi::{
	NtQuerySystemInformation, SystemProcessIdInformation, SYSTEM_PROCESS_ID_INFORMATION,
};

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
			// this can happen if OpenProcess failed with access denied error
			// or if DuplicateHandle() in SafeHandle clone() fails
			Err(ProcessError::AccessDenied { pid: self.pid })
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
			Err(WindowsOsError::Win32Error {
				code: ERROR_ACCESS_DENIED,
				..
			}) => {
				// this method is slower as it iterates over all processes
				// but it can bypass some security restrictions
				let process = ProcessInformation::get(pid)?;
				Ok(Process {
					pid: process.pid,
					// TODO: check if time is correct (should be counted from boot time)
					create_time: Duration::from_nanos(process.create_time_ns),
					busy: Duration::from_nanos(process.user_time_ns + process.kernel_time_ns),
					instant: Instant::now(),
					handle: SafeHandle::get_invalid(),
					access_rights: 0,
				})
			}
			Err(error) => Err(windows_error_to_process_error(pid, error)),
		}
	}

	pub(crate) fn sys_ppid(&self) -> ProcessResult<Option<Pid>> {
		todo!()
	}

	pub(crate) fn sys_name(&self) -> ProcessResult<String> {
		if self.pid == 0 {
			return Ok("System Idle Process".to_owned());
		} else if self.pid == 4 {
			return Ok("System".to_owned());
		}

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
			match self.get_handle() {
				Ok(handle) => {
					if QueryFullProcessImageNameW(
						handle.get_raw(),
						0,
						buffer.as_mut_ptr() as *mut u16,
						&mut size as *mut _,
					) == 0
					{
						// Attempt to query win32 path of WSL processes returns ERROR_GEN_FAILURE
						// use NT path
						let code = GetLastError();
						if code == ERROR_GEN_FAILURE
							&& QueryFullProcessImageNameW(
								handle.get_raw(),
								1, /* PROCESS_NAME_NATIVE */
								buffer.as_mut_ptr() as *mut u16,
								&mut size as *mut _,
							) != 0
						{
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
				Err(ProcessError::AccessDenied { .. }) => {
					const INITIAL_BUFFER_SIZE: USHORT = 80;
					let mut buffer: Vec<u16> = Vec::with_capacity(INITIAL_BUFFER_SIZE as usize);
					buffer.set_len(INITIAL_BUFFER_SIZE as usize);

					let mut status: i32;
					let mut spii: SYSTEM_PROCESS_ID_INFORMATION;
					loop {
						spii = SYSTEM_PROCESS_ID_INFORMATION {
							ProcessId: self.pid as _,
							ImageName: UNICODE_STRING {
								Buffer: buffer.as_mut_ptr(),
								Length: 0,
								MaximumLength: (buffer.len() * mem::size_of::<u16>()) as USHORT,
							},
						};

						status = NtQuerySystemInformation(
							SystemProcessIdInformation,
							&mut spii as *mut _ as *mut _,
							mem::size_of::<SYSTEM_PROCESS_ID_INFORMATION>() as u32,
							ptr::null_mut(),
						);

						if status == STATUS_INFO_LENGTH_MISMATCH {
							let mut new_size =
								spii.ImageName.MaximumLength as usize / mem::size_of::<u16>();
							let mut additional = new_size - buffer.len();

							if additional == 0 {
								// if running on 32-bit (at least under WoW64)
								// NtQuerySystemInformation does not return required length
								// on x64 seems to work fine but it is still kept just in case

								additional = buffer.len();
								new_size = buffer.len() + additional;
								buffer.reserve(additional);
								buffer.set_len(new_size);
								continue;
							} else {
								buffer.reserve(additional);
								buffer.set_len(new_size);

								spii.ImageName.Buffer = buffer.as_mut_ptr();
								spii.ImageName.MaximumLength =
									(buffer.len() * mem::size_of::<u16>()) as USHORT;

								status = NtQuerySystemInformation(
									SystemProcessIdInformation,
									&mut spii as *mut _ as *mut _,
									mem::size_of::<SYSTEM_PROCESS_ID_INFORMATION>() as u32,
									ptr::null_mut(),
								);
								break;
							}
						} else {
							break;
						}
					}

					if status != 0 {
						return Err(windows_error_to_process_error(
							self.pid,
							WindowsOsError::nt_error("NtQuerySystemInformation", status),
						));
					}

					if !self.is_still_running() {
						return Err(ProcessError::NoSuchProcess { pid: self.pid });
					}

					if spii.ImageName.Length == 0 || spii.ImageName.Buffer.is_null() {
						return Ok(PathBuf::new());
					}

					match String::from_utf16(&buffer[..spii.ImageName.Length as usize / 2]) {
						Ok(x) => {
							// TODO: convert to path to win32 format
							Ok(PathBuf::from(x))
						}
						Err(e) => Err(ProcessError::PsutilError {
							pid: self.pid,
							source: Error::FromUtf16ConvertError { source: e },
						}),
					}
				}
				Err(e) => Err(e),
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
		let data = self.get_process_data(ProcessDataKind::CWD)?;
		match String::from_utf16(&data) {
			Ok(x) => Ok(PathBuf::from(x)),
			Err(e) => Err(ProcessError::PsutilError {
				pid: self.pid,
				source: Error::FromUtf16ConvertError { source: e },
			}),
		}
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
		match self.get_handle() {
			Ok(handle) => {
				let mut creation_time: FILETIME = windows_filetime_default();
				let mut exit_time: FILETIME = windows_filetime_default();
				let mut kernel_time: FILETIME = windows_filetime_default();
				let mut user_time: FILETIME = windows_filetime_default();

				if unsafe {
					GetProcessTimes(
						handle.get_raw(),
						&mut creation_time as *mut _,
						&mut exit_time as *mut _,
						&mut kernel_time as *mut _,
						&mut user_time as *mut _,
					)
				} != 0
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
			Err(ProcessError::AccessDenied { .. }) => {
				let p = ProcessInformation::get(self.pid)?;
				if p.create_time_ns as u128 != self.create_time.as_nanos() {
					return Err(ProcessError::NoSuchProcess { pid: self.pid });
				}

				Ok(ProcessCpuTimes {
					user: Duration::from_nanos(p.user_time_ns),
					system: Duration::from_nanos(p.kernel_time_ns),
					// TODO:
					children_user: Duration::from_nanos(0),
					children_system: Duration::from_nanos(0),
				})
			}
			Err(e) => Err(e),
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

	pub(crate) fn get_process_data(&self, data_kind: ProcessDataKind) -> ProcessResult<Vec<u16>> {
		// Workaround for https://github.com/giampaolo/psutil/issues/875
		// based on https://github.com/giampaolo/psutil/blob/5be673aa2a0fccf079803bc2e3720fb46463793b/psutil/_pswindows.py#L685
		let mut delay = 1;
		for _ in 0..33 {
			match unsafe { self.get_process_data_internal(data_kind) } {
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
				Ok(x) => return Ok(x),
			};
		}

		Err(ProcessError::AccessDenied { pid: self.pid })
	}

	#[allow(non_snake_case)]
	unsafe fn get_process_data_internal(
		&self,
		data_kind: ProcessDataKind,
	) -> ProcessResult<Vec<u16>> {
		let handle = self.raise_privileges(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ)?;
		#[cfg(target_pointer_width = "64")]
		let mut ppeb32: *mut u8 = ptr::null_mut();

		let mut src: *const u8 = ptr::null();
		let mut size: usize = 0;

		#[cfg(target_pointer_width = "32")]
		let they_are_wow64: bool;
		#[cfg(target_pointer_width = "32")]
		let mut src64: u64 = 0;
		#[cfg(target_pointer_width = "64")]
		#[allow(non_upper_case_globals)]
		const src64: u64 = 0;

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
			let mut t = 0;

			if IsWow64Process(handle.get_raw(), &mut t as *mut _) == 0 {
				return Err(windows_error_to_process_error(
					self.pid,
					WindowsOsError::last_win32_error("IsWow64Process"),
				));
			}
			they_are_wow64 = t != 0;

			if WOW64.are_we_wow64 && !they_are_wow64 {
				if WOW64.NtWow64QueryInformationProcess64.is_none()
					|| WOW64.NtWow64ReadVirtualMemory64.is_none()
				{
					return Err(ProcessError::PsutilError {
						pid: self.pid,
						source: Error::OtherError {
							message: "can't query 64-bit process in 32-bit-WoW mode".to_string(),
						},
					});
				}

				let NtWow64QueryInformationProcess64 =
					WOW64.NtWow64QueryInformationProcess64.unwrap();
				let NtWow64ReadVirtualMemory64 = WOW64.NtWow64ReadVirtualMemory64.unwrap();

				let mut pbi64: PROCESS_BASIC_INFORMATION64 = mem::zeroed();
				let mut peb64: PEB64 = mem::zeroed();
				let mut procParameters64: RTL_USER_PROCESS_PARAMETERS64 = mem::zeroed();

				let mut status = (NtWow64QueryInformationProcess64)(
					handle.get_raw(),
					ProcessBasicInformation,
					&mut pbi64 as *mut _ as *mut _,
					mem::size_of::<PROCESS_BASIC_INFORMATION64>() as u32,
					ptr::null_mut(),
				);

				if status != 0 {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtWow64QueryInformationProcess64", status),
					));
				}

				status = (NtWow64ReadVirtualMemory64)(
					handle.get_raw(),
					pbi64.PebBaseAddress,
					&mut peb64 as *mut _ as *mut _,
					mem::size_of::<PEB64>() as u64,
					ptr::null_mut(),
				);

				if status != 0 {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtWow64ReadVirtualMemory64", status),
					));
				}

				status = (NtWow64ReadVirtualMemory64)(
					handle.get_raw(),
					peb64.ProcessParameters,
					&mut procParameters64 as *mut _ as *mut _,
					mem::size_of::<RTL_USER_PROCESS_PARAMETERS64>() as u64,
					ptr::null_mut(),
				);

				if status != 0 {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtWow64ReadVirtualMemory64", status),
					));
				}

				match data_kind {
					ProcessDataKind::CMDLINE => {
						src64 = procParameters64.CommandLine.Buffer;
						size = procParameters64.CommandLine.Length as usize;
					}
					ProcessDataKind::CWD => {
						src64 = procParameters64.CurrentDirectoryPath.Buffer;
						size = procParameters64.CurrentDirectoryPath.Length as usize;
					}
					ProcessDataKind::ENVIRONMENT => {
						src64 = procParameters64.env;
					}
				};
			}
		}

		#[cfg(target_pointer_width = "64")]
		type UintPtr = u64;
		#[cfg(target_pointer_width = "32")]
		type UintPtr = u32;

		if src.is_null() && src64 == 0 {
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
				if WOW64.are_we_wow64 && !they_are_wow64 {
					return Err(ProcessError::PsutilError {
						pid: self.pid,
						source: Error::OtherError {
							message: "can't query 64-bit process in 32-bit-WoW mode".to_string(),
						},
					});
				}
			}
			size = self.get_process_region_size(&handle, mem::transmute(src))?;
		}

		let mut buffer: Vec<u16> = Vec::with_capacity(size / 2 + 1);
		buffer.set_len(buffer.capacity());

		#[cfg(target_pointer_width = "32")]
		{
			if WOW64.are_we_wow64 && !they_are_wow64 {
				let status = (WOW64.NtWow64ReadVirtualMemory64.unwrap())(
					handle.get_raw(),
					src64,
					buffer.as_mut_ptr() as *mut c_void,
					size as u64,
					ptr::null_mut(),
				);

				if status != 0 {
					return Err(windows_error_to_process_error(
						self.pid,
						WindowsOsError::nt_error("NtWow64ReadVirtualMemory64", status),
					));
				}
				let buffer_len = buffer.len();
				buffer[buffer_len - 1] = 0;
				return Ok(buffer);
			}
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
		let src = src as UintPtr;
		let base_address = mbi.BaseAddress as UintPtr;

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
		self.get_process_data(ProcessDataKind::CMDLINE)
	}
	unsafe fn get_command_line(&self) -> ProcessResult<Vec<u16>> {
		if let Ok(r) = self.get_command_line_using_peb() {
			return Ok(r);
		}

		if !is_windows_version_higher_than_8_1() {
			// not supported on pre win 8.1
			return Err(ProcessError::PsutilError {
				pid: self.pid,
				source: Error::OtherError {
					message: "requires Windows 8.1+".to_string(),
				},
			});
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

	pub(crate) fn is_still_running(&self) -> bool {
		match self.get_handle() {
			Ok(handle) => {
				let status = unsafe { WaitForSingleObject(handle.get_raw(), 0) };
				status == WAIT_TIMEOUT
			}
			Err(_) => {
				if let Ok(processes) = ProcessInformationArray::get() {
					for process in processes {
						if process.pid == self.pid {
							return process.create_time_ns as u128 == self.create_time.as_nanos();
						}
					}
					false
				} else {
					false
				}
			}
		}
	}
}

pub fn processes() -> Result<Vec<ProcessResult<Process>>> {
	let processes = ProcessInformationArray::get()?;
	let mut v: Vec<ProcessResult<Process>> =
		Vec::with_capacity(processes.estimated_num_processes() as usize);
	for process in processes {
		let (handle, access_rights) = match unsafe { try_open_process_for_query(process.pid, true) }
		{
			Ok((handle, access_rights)) => {
				let mut creation_time: FILETIME = windows_filetime_default();
				let mut exit_time: FILETIME = windows_filetime_default();
				let mut kernel_time: FILETIME = windows_filetime_default();
				let mut user_time: FILETIME = windows_filetime_default();

				if unsafe {
					GetProcessTimes(
						handle.get_raw(),
						&mut creation_time as *mut _,
						&mut exit_time as *mut _,
						&mut kernel_time as *mut _,
						&mut user_time as *mut _,
					)
				} == 0 && windows_filetime_to_ns(&creation_time) != process.create_time_ns
				{
					// process is gone
					continue;
				}

				(handle, access_rights)
			}
			Err(e) => match e {
				WindowsOsError::Win32Error {
					code: ERROR_ACCESS_DENIED,
					..
				} => (SafeHandle::get_invalid(), 0),
				WindowsOsError::Win32Error {
					code: ERROR_INVALID_PARAMETER,
					..
				} => {
					// process is gone
					continue;
				}
				e => {
					v.push(Err(windows_error_to_process_error(process.pid, e)));
					continue;
				}
			},
		};

		v.push(Ok(Process {
			pid: process.pid,
			// TODO: check if time is correct (should be counted from boot time)
			create_time: Duration::from_nanos(process.create_time_ns),
			busy: Duration::from_nanos(process.user_time_ns + process.kernel_time_ns),
			instant: Instant::now(),
			handle,
			access_rights,
		}));
	}

	Ok(v)
}
