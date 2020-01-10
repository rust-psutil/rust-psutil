// https://github.com/heim-rs/heim/blob/master/heim-host/src/platform.rs
// Not found in python psutil.

use std::ffi::CStr;
use std::io;
use std::mem;
use std::str::FromStr;

use platforms::target::{Arch, OS};

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

pub fn info() -> io::Result<Info> {
    unsafe {
        let mut uts = mem::MaybeUninit::<libc::utsname>::uninit();
        let result = libc::uname(uts.as_mut_ptr());

        if result != 0 {
            Err(io::Error::last_os_error())
        } else {
            let uts = uts.assume_init();

            let raw_operating_system = CStr::from_ptr(uts.sysname.as_ptr())
                .to_string_lossy()
                .into_owned();
            let operating_system = OS::from_str(&raw_operating_system).unwrap_or(OS::Unknown);

            let release = CStr::from_ptr(uts.release.as_ptr())
                .to_string_lossy()
                .into_owned();
            let version = CStr::from_ptr(uts.version.as_ptr())
                .to_string_lossy()
                .into_owned();
            let hostname = CStr::from_ptr(uts.nodename.as_ptr())
                .to_string_lossy()
                .into_owned();

            let raw_architecture = CStr::from_ptr(uts.machine.as_ptr()).to_string_lossy();
            let architecture = Arch::from_str(&raw_architecture).unwrap_or(Arch::Unknown);

            Ok(Info {
                operating_system,
                release,
                version,
                hostname,
                architecture,
            })
        }
    }
}
