// https://github.com/heim-rs/heim/blob/master/heim-host/src/platform.rs

use std::str::FromStr;

use nix::sys;
use platforms::target::{Arch, OS};

use crate::host::Info;

pub fn info() -> Info {
	let utsname = sys::utsname::uname().unwrap();

	let operating_system = utsname
		.sysname()
		.to_str()
		.and_then(|s| OS::from_str(s).ok())
		.unwrap_or(OS::Unknown);
	let release = utsname.release().to_str().unwrap_or_default().to_string();
	let version = utsname.version().to_str().unwrap_or_default().to_string();
	let hostname = utsname.nodename().to_str().unwrap_or_default().to_string();
	let architecture = utsname
		.machine()
		.to_str()
		.and_then(|s| Arch::from_str(s).ok())
		.unwrap_or(Arch::Unknown);

	Info {
		operating_system,
		release,
		version,
		hostname,
		architecture,
	}
}
