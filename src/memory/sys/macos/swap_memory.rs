// https://github.com/heim-rs/heim/blob/master/heim-memory/src/sys/macos/swap.rs
// https://github.com/heim-rs/heim/blob/master/heim-memory/src/sys/macos/bindings.rs
// https://github.com/heim-rs/heim/blob/master/heim-common/src/sys/macos/mod.rs

use std::io;
use std::mem;
use std::ptr;

use nix::libc;

use super::common;
use crate::PAGE_SIZE;
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

const CTL_VM: libc::c_int = 2;
const VM_SWAPUSAGE: libc::c_int = 5;

unsafe fn vm_swapusage() -> io::Result<libc::xsw_usage> {
	let mut name: [i32; 2] = [CTL_VM, VM_SWAPUSAGE];
	let mut value = mem::MaybeUninit::<libc::xsw_usage>::uninit();
	let mut length = mem::size_of::<libc::xsw_usage>();

	let result = libc::sysctl(
		name.as_mut_ptr(),
		2,
		value.as_mut_ptr() as *mut libc::c_void,
		&mut length,
		ptr::null_mut(),
		0,
	);

	if result == 0 {
		let value = value.assume_init();
		Ok(value)
	} else {
		Err(io::Error::last_os_error())
	}
}

pub fn swap_memory() -> io::Result<SwapMemory> {
	let xsw_usage = unsafe { vm_swapusage()? };
	let vm_stats = unsafe { common::host_vm_info()? };
	let page_size = *PAGE_SIZE;

	let total = u64::from(xsw_usage.xsu_total);
	let used = u64::from(xsw_usage.xsu_used);
	let free = u64::from(xsw_usage.xsu_avail);
	let swapped_in = u64::from(vm_stats.pageins) * page_size;
	let swapped_out = u64::from(vm_stats.pageouts) * page_size;

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
