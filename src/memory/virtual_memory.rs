use crate::{Bytes, Percent};

#[derive(Debug, Clone)]
pub struct VirtualMemory {
	pub(crate) total: Bytes,
	pub(crate) available: Bytes,
	pub(crate) percent: Percent,
	pub(crate) used: Bytes,
	pub(crate) free: Bytes,
	pub(crate) active: Bytes,
	pub(crate) inactive: Bytes,
	pub(crate) buffers: Bytes,
	pub(crate) cached: Bytes,
	pub(crate) shared: Bytes,
}

impl VirtualMemory {
	/// Amount of total memory.
	pub fn total(&self) -> Bytes {
		self.total
	}

	/// Amount of memory available for new processes.
	pub fn available(&self) -> Bytes {
		self.available
	}

	/// Memory currently in use.
	pub fn used(&self) -> Bytes {
		self.used
	}

	/// Memory not being used.
	pub fn free(&self) -> Bytes {
		self.free
	}

	/// New method, not in Python psutil.
	/// Percent of memory used.
	pub fn percent(&self) -> Percent {
		self.percent
	}
}
