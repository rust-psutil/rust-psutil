use nix::libc::{c_void, sysctl, timeval};
use std::time::Duration;
use std::{mem, ptr};

use crate::{Error, Result};

/// New function, not in Python psutil.
pub fn uptime() -> Result<Duration> {
	let mut data: timeval = unsafe { mem::zeroed() };
	let mib = [1, 21];
	unsafe {
		sysctl(
			&mib[0] as *const _ as *mut _,
			mib.len() as u32,
			&mut data as *mut _ as *mut c_void,
			&mut mem::size_of::<timeval>(),
			ptr::null_mut(),
			0,
		);
	}
	Ok(Duration::from_secs(data.tv_sec as u64))
}
