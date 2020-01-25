use std::fs;
use std::io;

use crate::memory::make_map;
use crate::memory::SwapMemory;
use crate::utils::not_found;

// TODO: return an option for when swap is disabled?
pub fn swap_memory() -> io::Result<SwapMemory> {
	let data = fs::read_to_string("/proc/meminfo")?;
	let meminfo = make_map(&data)?;

	let data = fs::read_to_string("/proc/vmstat")?;
	let vmstat = make_map(&data)?;

	let total = *meminfo
		.get("SwapTotal:")
		.ok_or_else(|| not_found("SwapTotal"))?;
	let free = *meminfo
		.get("SwapFree:")
		.ok_or_else(|| not_found("SwapFree"))?;

	let swapped_in = *vmstat.get("pswpin").ok_or_else(|| not_found("pswpin"))?;
	let swapped_out = *vmstat.get("pswpout").ok_or_else(|| not_found("pswpout"))?;

	let used = total - free;
	// total will be 0 if swap is disabled
	let percent = if total == 0 {
		0.0
	} else {
		((used as f64 / total as f64) * 100.0) as f32
	};

	Ok(SwapMemory {
		total,
		used,
		free,
		percent,
		swapped_in,
		swapped_out,
	})
}
