use std::fs;
use std::io;

use crate::memory::make_map;
use crate::memory::VirtualMemory;
use crate::utils::not_found;

pub fn virtual_memory() -> io::Result<VirtualMemory> {
	let data = fs::read_to_string("/proc/meminfo")?;
	let mem_info = make_map(&data)?;

	let total = *mem_info
		.get("MemTotal:")
		.ok_or_else(|| not_found("MemTotal"))?;
	let free = *mem_info
		.get("MemFree:")
		.ok_or_else(|| not_found("MemFree"))?;
	let buffers = *mem_info
		.get("Buffers:")
		.ok_or_else(|| not_found("Buffers"))?;
	let cached = *mem_info.get("Cached:").ok_or_else(|| not_found("Cached"))?
        // "free" cmdline utility sums reclaimable to cached.
        // Older versions of procps used to add slab memory instead.
        // This got changed in:
        //  https://gitlab.com/procps-ng/procps/commit/05d751c4f076a2f0118b914c5e51cfbb4762ad8e
        + *mem_info
            .get("SReclaimable:")
            .ok_or_else(|| not_found("SReclaimable"))?; // since Linux 2.6.19
	let active = *mem_info.get("Active:").ok_or_else(|| not_found("Active"))?;
	let inactive = *mem_info
		.get("Inactive:")
		.ok_or_else(|| not_found("Inactive"))?;
	// since Linux 3.14
	let available = *mem_info
		.get("MemAvailable:")
		.ok_or_else(|| not_found("MemAvailable"))?;
	// since Linux 2.6.32
	let shared = *mem_info.get("Shmem:").ok_or_else(|| not_found("Shmem"))?;

	let used = total - free - cached - buffers;
	let percent = (((total as f64 - available as f64) / total as f64) * 100.0) as f32;

	Ok(VirtualMemory {
		total,
		available,
		shared,
		free,
		buffers,
		cached,
		active,
		inactive,
		used,
		percent,
	})
}
