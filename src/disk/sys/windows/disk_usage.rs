use std::iter::once;
use std::mem::{transmute, zeroed};
use std::os::windows::ffi::OsStrExt as _;
use std::path::Path;
use std::ptr;

use winapi::shared::ntdef::ULARGE_INTEGER;
use winapi::um::fileapi::GetDiskFreeSpaceExW;

use crate::utils::u64_percent;
use crate::windows_util::*;
use crate::{Bytes, Error, Percent, Result, WindowsOsError};

#[derive(Clone, Debug, Default)]
pub struct DiskUsage {
	pub(crate) total: Bytes,
	pub(crate) used: Bytes,
	pub(crate) free: Bytes,
	pub(crate) percent: Percent,
}

impl DiskUsage {
	/// Total disk size in bytes.
	pub fn total(&self) -> Bytes {
		self.total
	}

	/// Number of bytes used.
	pub fn used(&self) -> Bytes {
		self.used
	}

	/// Number of bytes free.
	pub fn free(&self) -> Bytes {
		self.free
	}

	/// Percentage of disk used.
	pub fn percent(&self) -> Percent {
		self.percent
	}
}

pub fn disk_usage<P>(path: P) -> Result<DiskUsage>
where
	P: AsRef<Path>,
{
	let raw_path: Vec<u16> = path
		.as_ref()
		.as_os_str()
		.encode_wide()
		.chain(once(0))
		.collect();
	unsafe {
		let mut total_uli: ULARGE_INTEGER = zeroed();
		let mut free_uli: ULARGE_INTEGER = zeroed();

		if GetDiskFreeSpaceExW(
			transmute(raw_path.as_ptr()),
			ptr::null_mut(),
			&mut total_uli as *mut _,
			&mut free_uli as *mut _,
		) == 0
		{
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GetDiskFreeSpaceExW",
			)));
		}
		let total = ularge_integer_to_u64(&total_uli);
		let free = ularge_integer_to_u64(&free_uli);

		Ok(DiskUsage {
			total,
			used: total - free,
			free: total,
			percent: u64_percent(total - free, total),
		})
	}
}
