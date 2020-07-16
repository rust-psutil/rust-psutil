use crate::memory::SwapMemory;
use crate::utils::u64_percent;
use crate::{Error, Result, WindowsOsError};
use std::mem::{size_of, MaybeUninit};

use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, LPMEMORYSTATUSEX, MEMORYSTATUSEX};

pub fn swap_memory() -> Result<SwapMemory> {
	unsafe {
		let mut msx: MEMORYSTATUSEX = MaybeUninit::uninit().assume_init();
		msx.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
		if GlobalMemoryStatusEx(&mut msx as LPMEMORYSTATUSEX) == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GlobalMemoryStatusEx",
			)));
		}

		Ok(SwapMemory {
			total: msx.ullTotalPageFile,
			used: msx.ullTotalPageFile - msx.ullAvailPageFile,
			free: msx.ullAvailPageFile,
			percent: if msx.ullTotalPageFile != 0 {
				u64_percent(
					msx.ullTotalPageFile - msx.ullAvailPageFile,
					msx.ullTotalPageFile,
				)
			} else {
				0f32
			},
			// TODO:
			swapped_in: 0,
			swapped_out: 0,
		})
	}
}
