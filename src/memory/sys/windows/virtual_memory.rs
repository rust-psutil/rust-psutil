use crate::memory::VirtualMemory;
use crate::windows_util::global_memory_status_ex;
use crate::Result;

pub fn virtual_memory() -> Result<VirtualMemory> {
	let msx = global_memory_status_ex()?;

	Ok(VirtualMemory {
		total: msx.ullTotalPhys,
		available: msx.ullAvailPhys,
		used: msx.ullTotalPhys - msx.ullAvailPhys,
		free: msx.ullAvailPhys,
		percent: msx.dwMemoryLoad as f32,
	})
}
