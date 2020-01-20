// https://github.com/heim-rs/heim/blob/master/heim-host/src/platform.rs

use std::str::FromStr;

use nix::sys;

use platforms::target::{Arch, OS};

/// Not found in Python psutil.
#[derive(Clone, Debug)]
pub struct Info {
	operating_system: OS,
	release: String,
	version: String,
	hostname: String,
	architecture: Arch,
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

pub fn info() -> Info {
	let utsname = sys::utsname::uname();

	let operating_system = OS::from_str(utsname.sysname()).unwrap_or(OS::Unknown);
	let release = utsname.release().to_string();
	let version = utsname.version().to_string();
	let hostname = utsname.nodename().to_string();
	let architecture = Arch::from_str(utsname.machine()).unwrap_or(Arch::Unknown);

	Info {
		operating_system,
		release,
		version,
		hostname,
		architecture,
	}
}
