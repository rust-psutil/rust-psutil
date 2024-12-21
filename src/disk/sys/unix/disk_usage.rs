use std::path::Path;

use nix::sys;

use crate::{Bytes, Percent, Result};

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

	/// Number of bytes used in Disk.
	pub fn used(&self) -> Bytes {
		self.used
	}

	/// Number of bytes free in Disk.
	pub fn free(&self) -> Bytes {
		self.free
	}

	/// Percentage of disk used.
	pub fn percent(&self) -> Percent {
		self.percent
	}
}

// Disable the unnecessary_cast lint, as we need to do the u64 casts since on some platforms the
// types are aliased to u32.
// See https://github.com/rust-psutil/rust-psutil/issues/64 and
// https://github.com/rust-psutil/rust-psutil/pull/39
#[allow(clippy::unnecessary_cast)]
pub fn disk_usage<P>(path: P) -> Result<DiskUsage>
where
	P: AsRef<Path>,
{
	let statvfs = sys::statvfs::statvfs(path.as_ref())?;

	let total = statvfs.blocks() as u64 * statvfs.fragment_size() as u64;

	let avail_to_root = statvfs.blocks_free() as u64 * statvfs.fragment_size() as u64;
	let used = total - avail_to_root;

	let free = statvfs.blocks_available() as u64 * statvfs.fragment_size() as u64;

	let total_user = used + free;
	let percent = ((used as f64 / total_user as f64) * 100.0) as f32;

	Ok(DiskUsage {
		total,
		used,
		free,
		percent,
	})
}
