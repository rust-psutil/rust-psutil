use std::path::{Path, PathBuf};

use crate::disk::FileSystem;

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
