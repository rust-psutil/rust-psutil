use crate::cpu::CpuTimes;
use crate::windows_util::windows_filetime_to_ns;
use crate::{Error, Result, WindowsOsError};
use std::mem::{size_of, transmute, zeroed, MaybeUninit};
use std::ptr;
use std::time::Duration;

use winapi::shared::minwindef::FILETIME;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::processthreadsapi::GetSystemTimes;
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};

use ntapi::ntexapi::{
	NtQuerySystemInformation, SystemProcessorPerformanceInformation,
	SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION,
};

use crate::windows_util::*;

pub fn cpu_times() -> Result<CpuTimes> {
	unsafe {
		let mut idle: FILETIME = MaybeUninit::uninit().assume_init();
		let mut kernel: FILETIME = MaybeUninit::uninit().assume_init();
		let mut user: FILETIME = MaybeUninit::uninit().assume_init();

		if GetSystemTimes(
			&mut idle as *mut _,
			&mut kernel as *mut _,
			&mut user as *mut _,
		) == 0
		{
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GetSystemTimes",
			)));
		}

		let idle = windows_filetime_to_ns(&idle);

		return Ok(CpuTimes {
			idle: Duration::from_nanos(idle),
			system: Duration::from_nanos(windows_filetime_to_ns(&kernel) - idle),
			user: Duration::from_nanos(windows_filetime_to_ns(&user)),
			nice: Duration::from_nanos(0),
		});
	}
}

pub fn cpu_times_percpu() -> Result<Vec<CpuTimes>> {
	unsafe {
		let mut sysinfo: SYSTEM_INFO = zeroed();
		GetSystemInfo(&mut sysinfo as *mut _);
		let num_cpus = sysinfo.dwNumberOfProcessors as u32;

		let mut sppi: Vec<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION> =
			Vec::with_capacity(num_cpus as usize);
		sppi.set_len(sppi.capacity());

		let status = NtQuerySystemInformation(
			SystemProcessorPerformanceInformation,
			transmute(sppi.as_mut_ptr()),
			sppi.len() as u32 * size_of::<SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION>() as u32,
			ptr::null_mut(),
		);
		if status != STATUS_SUCCESS {
			return Err(Error::from(WindowsOsError::nt_error(
				"NtQuerySystemInformation",
				status,
			)));
		}

		let mut cpu_times: Vec<CpuTimes> = Vec::with_capacity(num_cpus as usize);
		for i in sppi.iter() {
			let user = large_integer_to_u64(&i.UserTime)
				.checked_mul(100)
				.unwrap_or(u64::MAX);
			let kernel = large_integer_to_u64(&i.KernelTime)
				.checked_mul(100)
				.unwrap_or(u64::MAX);
			let idle = large_integer_to_u64(&i.IdleTime)
				.checked_mul(100)
				.unwrap_or(u64::MAX);

			cpu_times.push(CpuTimes {
				idle: Duration::from_nanos(idle),
				system: Duration::from_nanos(kernel - idle),
				user: Duration::from_nanos(user),
				nice: Duration::from_nanos(0),
			});
		}

		return Ok(cpu_times);
	}
}
