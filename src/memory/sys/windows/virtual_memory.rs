use crate::memory::VirtualMemory;
use crate::{Error, Result, WindowsOsError};
use std::mem::{size_of, MaybeUninit};

use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, LPMEMORYSTATUSEX, MEMORYSTATUSEX};

pub fn virtual_memory() -> Result<VirtualMemory> {
	unsafe {
		let mut msx: MEMORYSTATUSEX = MaybeUninit::uninit().assume_init();
		msx.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
		if GlobalMemoryStatusEx(&mut msx as LPMEMORYSTATUSEX) == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GlobalMemoryStatusEx",
			)));
		}

		return Ok(VirtualMemory {
			total: msx.ullTotalPhys,
			available: msx.ullAvailPhys,
			used: msx.ullTotalPhys - msx.ullAvailPhys,
			free: msx.ullAvailPhys,
			percent: msx.dwMemoryLoad as f32,
		});
	}
}
