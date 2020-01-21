use std::fs;
use std::io;

use crate::memory::make_map;
use crate::utils::not_found;
use crate::{Bytes, Percent};

#[derive(Debug, Clone)]
pub struct SwapMemory {
	pub(crate) total: Bytes,
	pub(crate) used: Bytes,
	pub(crate) free: Bytes,
	pub(crate) percent: Percent,
	pub(crate) swapped_in: Bytes,
	pub(crate) swapped_out: Bytes,
}

impl SwapMemory {
	/// Amount of total swap memory.
	pub fn total(&self) -> Bytes {
		self.total
	}

	/// Amount of used swap memory.
	pub fn used(&self) -> Bytes {
		self.used
	}

	/// Amount of free swap memory.
	pub fn free(&self) -> Bytes {
		self.free
	}

	/// Percent of swap memory used.
	pub fn percent(&self) -> Percent {
		self.percent
	}

	/// Amount of memory swapped in from disk.
	/// Renamed from `sin` in Python psutil.
	pub fn swapped_in(&self) -> Bytes {
		self.swapped_in
	}

	/// Amount of memory swapped to disk.
	/// Renamed from `sout` in Python psutil.
	pub fn swapped_out(&self) -> Bytes {
		self.swapped_out
	}
}

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
	let percent = ((used as f64 / total as f64) * 100.0) as f32;

	Ok(SwapMemory {
		total,
		used,
		free,
		percent,
		swapped_in,
		swapped_out,
	})
}
