use std::mem::{self, MaybeUninit};
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::winerror::ERROR_UNRECOGNIZED_VOLUME;
use winapi::um::fileapi::{
	GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
	GetVolumeNameForVolumeMountPointW,
};
use winapi::um::winbase::{DRIVE_NO_ROOT_DIR, DRIVE_RAMDISK, DRIVE_REMOTE, DRIVE_UNKNOWN};
use winapi::um::winnt::{
	FILE_PERSISTENT_ACLS, FILE_READ_ONLY_VOLUME, FILE_SUPPORTS_EXTENDED_ATTRIBUTES,
	FILE_VOLUME_IS_COMPRESSED,
};

use crate::disk::{FileSystem, Partition};
use crate::{Error, Result, WindowsOsError};

unsafe fn get_partition(root: &[u16]) -> Result<Option<Partition>> {
	let mut fs_type_buffer: [MaybeUninit<u16>; MAX_PATH] = MaybeUninit::uninit().assume_init();
	let mut volume_name_buffer: [MaybeUninit<u16>; MAX_PATH] = MaybeUninit::uninit().assume_init();
	let mut volume_path_buffer: [MaybeUninit<u16>; MAX_PATH] = MaybeUninit::uninit().assume_init();
	fs_type_buffer[0] = MaybeUninit::new(0);
	volume_name_buffer[0] = MaybeUninit::new(0);
	volume_path_buffer[0] = MaybeUninit::new(0);

	if root.is_empty() {
		return Ok(None);
	}
	let drive_type_os = GetDriveTypeW(root.as_ptr());
	if drive_type_os == DRIVE_UNKNOWN
		|| drive_type_os == DRIVE_NO_ROOT_DIR
		|| drive_type_os == DRIVE_RAMDISK
		|| drive_type_os == DRIVE_REMOTE
	{
		return Ok(None);
	}

	let mut flags: u32 = 0;
	let mut fs_flags_array: Vec<&str> = Vec::new();

	if GetVolumeInformationW(
		root.as_ptr(),
		volume_name_buffer.as_mut_ptr() as *mut u16,
		volume_name_buffer.len() as u32,
		ptr::null_mut(),
		ptr::null_mut(),
		&mut flags as *mut _,
		fs_type_buffer.as_mut_ptr() as *mut u16,
		fs_type_buffer.len() as u32,
	) != 0
	{
		fs_flags_array.push(if flags & FILE_READ_ONLY_VOLUME == FILE_READ_ONLY_VOLUME {
			"ro"
		} else {
			"rw"
		});
		if flags & FILE_VOLUME_IS_COMPRESSED == FILE_VOLUME_IS_COMPRESSED {
			fs_flags_array.push("compressed");
		}
		fs_flags_array.push(if flags & FILE_PERSISTENT_ACLS == FILE_PERSISTENT_ACLS {
			"acl"
		} else {
			"noacl"
		});
		if flags & FILE_SUPPORTS_EXTENDED_ATTRIBUTES == FILE_SUPPORTS_EXTENDED_ATTRIBUTES {
			fs_flags_array.push("xattr");
		}
	} else {
		let e = WindowsOsError::last_win32_error_code();
		// thrown when Windows can't recognize file system
		// eg. when usb disk with unknown or no file system is attached
		if e == ERROR_UNRECOGNIZED_VOLUME {
		} else {
			return Err(Error::from(WindowsOsError::from_code(
				e,
				"GetVolumeInformationW",
			)));
		}
	}

	if GetVolumeNameForVolumeMountPointW(
		root.as_ptr(),
		volume_path_buffer.as_mut_ptr() as *mut u16,
		volume_path_buffer.len() as u32,
	) == 0
	{
		// some drivers don't support this call
		// assume empty volume name
		volume_path_buffer[0] = MaybeUninit::new(0);
	}

	let fs_name_terminator = fs_type_buffer
		.iter()
		.enumerate()
		.find(|(_, x)| mem::transmute::<MaybeUninit<u16>, u16>(**x) == 0u16)
		.map(|(i, _)| i)
		.unwrap_or(fs_type_buffer.len());

	let volume_name_terminator = volume_name_buffer
		.iter()
		.enumerate()
		.find(|(_, x)| mem::transmute::<MaybeUninit<u16>, u16>(**x) == 0u16)
		.map(|(i, _)| i)
		.unwrap_or(volume_name_buffer.len());

	let volume_path_terminator = volume_path_buffer
		.iter()
		.enumerate()
		.find(|(_, x)| mem::transmute::<MaybeUninit<u16>, u16>(**x) == 0u16)
		.map(|(i, _)| i)
		.unwrap_or(volume_path_buffer.len());

	let root_utf8 = String::from_utf16(root)?;
	let fs_utf8 = String::from_utf16(
		&*(&fs_type_buffer[..fs_name_terminator] as *const [std::mem::MaybeUninit<u16>]
			as *const [u16]),
	)?;
	let volume_path_utf8 = String::from_utf16(
		&*(&volume_path_buffer[..volume_path_terminator] as *const [std::mem::MaybeUninit<u16>]
			as *const [u16]),
	)?;

	let volume_name_utf8 = String::from_utf16(
		&*(&volume_name_buffer[..volume_name_terminator] as *const [std::mem::MaybeUninit<u16>]
			as *const [u16]),
	)?;

	Ok(Some(Partition {
		device: volume_path_utf8,
		mountpoint: PathBuf::from(root_utf8),
		filesystem: FileSystem::from_str(&fs_utf8).unwrap_or(FileSystem::Other(fs_utf8)),
		mount_options: fs_flags_array.join(","),
		name: if volume_name_utf8.is_empty() {
			None
		} else {
			Some(volume_name_utf8)
		},
	}))
}

pub fn partitions() -> Result<Vec<Partition>> {
	unsafe {
		let mut buffer: Vec<u16> = Vec::with_capacity(255);
		buffer.set_len(buffer.capacity());

		let mut n = GetLogicalDriveStringsW(buffer.len() as u32 - 1, buffer.as_mut_ptr());
		if n == 0 {
			return Err(Error::from(WindowsOsError::last_win32_error(
				"GetLogicalDriveStringsW",
			)));
		}

		if n > buffer.len() as u32 {
			n = buffer.len() as u32;
		}
		let s = &buffer[..n as usize];

		let mut partitions: Vec<Partition> = Vec::new();

		#[allow(unused_mut)]
		let mut last_error: Option<Error> = None;

		for root in s.split(|x| *x == 0) {
			match get_partition(root) {
				Ok(s) => {
					if let Some(p) = s {
						partitions.push(p)
					}
				}
				Err(e) => {
					last_error = Some(e);
				}
			}
		}

		if let Some(error) = last_error {
			if partitions.is_empty() {
				return Err(error);
			}
		}

		Ok(partitions)
	}
}
