use crate::memory::SwapMemory;
use crate::utils::u64_percent;
use crate::windows_util::global_memory_status_ex;
use crate::Result;

pub fn swap_memory() -> Result<SwapMemory> {
	let msx = global_memory_status_ex()?;

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
