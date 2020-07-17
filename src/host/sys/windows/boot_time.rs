use crate::windows_util::{filetime_to_unix_time, windows_filetime_default};
use crate::Result;
use std::time::{Duration, SystemTime};

use winapi::shared::minwindef::FILETIME;
use winapi::um::sysinfoapi::{GetSystemTimeAsFileTime, GetTickCount64};

pub fn boot_time() -> Result<SystemTime> {
	let mut ft: FILETIME = windows_filetime_default();
	unsafe { GetSystemTimeAsFileTime(&mut ft as *mut _) };

	let unix_time = filetime_to_unix_time(&ft) - (unsafe { GetTickCount64() } / 1000u64);

	Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(unix_time))
}
