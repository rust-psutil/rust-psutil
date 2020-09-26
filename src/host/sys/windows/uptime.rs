use std::time::Duration;

use winapi::um::sysinfoapi::GetTickCount64;

use crate::Result;

pub fn uptime() -> Result<Duration> {
	Ok(Duration::from_millis(unsafe { GetTickCount64() }))
}
