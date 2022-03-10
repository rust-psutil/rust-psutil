// https://github.com/heim-rs/heim/blob/master/heim-host/src/platform.rs

use std::str::FromStr;

use nix::sys;
use platforms::target::{Arch, OS};

use crate::host::Info;

pub fn info() -> Info {
	let utsname = sys::utsname::uname();

	let operating_system = match utsname.sysname().to_lowercase().as_str() {
		"darwin" => OS::MacOS,
		s => OS::from_str(s).unwrap_or(OS::Unknown),
	};

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
