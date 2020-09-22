#[cfg(not(target_os = "macos"))]
use std::time::SystemTime;

#[cfg(not(target_os = "macos"))]
use chrono::prelude::*;

use psutil::host;

fn main() {
	let info = host::info();
	println!("OS        : {:?}", info.operating_system());
	println!("Version   : {}", info.version());
	println!("Release   : {}", info.release());
	println!("Host Name : {}", info.hostname());
	println!("CPU       : {:?}", info.architecture());

	// TODO: support this on macos
	#[cfg(not(target_os = "macos"))]
	{
		let uptime = host::uptime().unwrap();
		let seconds_total = uptime.as_secs();
		let days = seconds_total / 86400;
		let hours = (seconds_total - (days * 86400)) / 3600;
		let minutes = (seconds_total - (days * 86400) - (hours * 3600)) / 60;
		println!(
			"Uptime    : {} days, {} hours, {} minutes",
			days, hours, minutes
		);

		let boot_time = host::boot_time().unwrap();
		let d = boot_time
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		let utc: DateTime<Utc> =
			DateTime::from_utc(NaiveDateTime::from_timestamp(d as i64, 0), Utc);
		let local = utc.with_timezone(&Local);
		println!("Boot Time : {}", local.format("%Y-%m-%d %H:%M:%S"));
	}
}
