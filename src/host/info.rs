use platforms::target::{Arch, OS};

/// Not found in Python psutil.
#[derive(Clone, Debug)]
pub struct Info {
	pub(crate) operating_system: OS,
	pub(crate) release: String,
	pub(crate) version: String,
	pub(crate) hostname: String,
	pub(crate) architecture: Arch,
}

impl Info {
	pub fn operating_system(&self) -> OS {
		self.operating_system
	}

	pub fn release(&self) -> &str {
		&self.release
	}

	pub fn version(&self) -> &str {
		&self.version
	}

	pub fn hostname(&self) -> &str {
		&self.hostname
	}

	pub fn architecture(&self) -> Arch {
		self.architecture
	}
}
