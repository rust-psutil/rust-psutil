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
