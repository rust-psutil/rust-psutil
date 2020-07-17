use crate::disk::DiskIoCounters;
use crate::disk::{partitions_physical, Partition};
use crate::windows_util::*;
use crate::{Error, Result, WindowsOsError};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::{size_of, transmute, zeroed};
use std::os::windows::ffi::OsStrExt as _;
use std::ptr;
use std::time::Duration;

use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{DISK_PERFORMANCE, IOCTL_DISK_PERFORMANCE};
use winapi::um::winnt::{
	FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, STANDARD_RIGHTS_READ,
};

unsafe fn get_io_counter_for_partition(partition: &Partition) -> Result<DiskIoCounters> {
	let device = partition.device().trim_end_matches('\\');
	let device_utf16: Vec<u16> = OsStr::new(device).encode_wide().chain(once(0)).collect();

	let handle = SafeHandle::from_raw_handle(CreateFileW(
		transmute(device_utf16.as_ptr()),
		STANDARD_RIGHTS_READ,
		FILE_SHARE_READ | FILE_SHARE_WRITE,
		ptr::null_mut(),
		OPEN_EXISTING,
		FILE_ATTRIBUTE_NORMAL,
		ptr::null_mut(),
	));
	if !handle.is_valid() {
		return Err(Error::from(WindowsOsError::last_win32_error("CreateFileW")));
	}

	let mut stat: DISK_PERFORMANCE = zeroed();
	let mut bytes_returned: u32 = 0;

	if DeviceIoControl(
		handle.get_raw(),
		IOCTL_DISK_PERFORMANCE,
		ptr::null_mut(),
		0,
		&mut stat as *mut _ as *mut std::ffi::c_void,
		size_of::<DISK_PERFORMANCE>() as u32,
		&mut bytes_returned as *mut _,
		ptr::null_mut(),
	) == 0
	{
		return Err(Error::from(WindowsOsError::last_win32_error(
			"DeviceIoControl",
		)));
	}

	Ok(DiskIoCounters {
		read_count: stat.ReadCount as u64,
		write_count: stat.WriteCount as u64,
		read_bytes: large_integer_to_u64(&stat.BytesRead),
		write_bytes: large_integer_to_u64(&stat.BytesWritten),
		read_time: Duration::from_millis(large_integer_to_u64(&stat.ReadTime) / 10000000),
		write_time: Duration::from_millis(large_integer_to_u64(&stat.WriteTime) / 10000000),
	})
}

pub(crate) fn disk_io_counters_per_partition() -> Result<HashMap<String, DiskIoCounters>> {
	let p = partitions_physical()?;

	let mut map: HashMap<String, DiskIoCounters> = HashMap::new();
	let mut last_error: Option<Error> = None;

	for p in p.iter() {
		match unsafe { get_io_counter_for_partition(p) } {
			Ok(counters) => {
				map.insert(p.device().to_owned(), counters);
			}
			Err(e) => last_error = Some(e),
		};
	}

	if map.is_empty() {
		if let Some(e) = last_error {
			return Err(e);
		}
	}

	Ok(map)
}
