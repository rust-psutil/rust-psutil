use std::path::{Path, PathBuf};

use crate::disk::{partitions, FileSystem};
use crate::Result;

#[derive(Clone, Debug)]
pub struct Partition {
	pub(crate) device: String,
	pub(crate) mountpoint: PathBuf,
	pub(crate) filesystem: FileSystem,
	pub(crate) mount_options: String,
}

impl Partition {
	pub fn device(&self) -> &str {
		&self.device
	}

	pub fn mountpoint(&self) -> &Path {
		&self.mountpoint
	}

	/// Renamed from `fstype` in Python psutil.
	pub fn filesystem(&self) -> &FileSystem {
		&self.filesystem
	}

	/// Renamed from `opts` in Python psutil.
	pub fn mount_options(&self) -> &str {
		&self.mount_options
	}
}

pub fn partitions_physical() -> Result<Vec<Partition>> {
	Ok(partitions()?
		.into_iter()
		.filter(|partition| partition.filesystem.is_physical())
		.collect())
}
