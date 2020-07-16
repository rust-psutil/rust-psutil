use crate::host::Info;
//use crate::{Error, WindowsOsError};
use platforms::target::{Arch, OS};

use std::iter::once;
use std::mem::{zeroed, MaybeUninit};
use std::ptr;

use winapi::shared::{
	minwindef::HKEY,
	winerror::{ERROR_MORE_DATA, ERROR_SUCCESS},
};
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
use winapi::um::winbase::GetComputerNameW;
use winapi::um::winnt::{
	KEY_QUERY_VALUE, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM,
	PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_IA64, PROCESSOR_ARCHITECTURE_INTEL,
	PROCESSOR_ARCHITECTURE_UNKNOWN, REG_SZ,
};
use winapi::um::winreg::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE};

unsafe fn get_string_value(key: HKEY, buffer: &mut Vec<u16>, n: &str) -> Option<String> {
	let mut key_type: u32 = 0;
	let mut data_len: u32 = 0;
	for _ in 0..5 {
		let status = RegQueryValueExW(
			key,
			n.encode_utf16()
				.chain(once(0))
				.collect::<Vec<u16>>()
				.as_ptr(),
			ptr::null_mut(),
			&mut key_type as *mut _,
			buffer.as_mut_ptr() as *mut u8,
			&mut data_len as *mut _,
		) as u32;
		if status == ERROR_MORE_DATA {
			buffer.reserve(data_len as usize);
			buffer.set_len(buffer.capacity());
			continue;
		} else if status == ERROR_SUCCESS {
			if key_type == REG_SZ {
				if let Ok(x) = String::from_utf16(&buffer[..(data_len / 2) as usize]) {
					return Some(x.trim_end_matches(|x| x == '\x00').to_string());
				}
			}
			break;
		} else {
			break;
		}
	}

	return None;
}

unsafe fn get_version_and_release(buffer: &mut Vec<u16>) -> (String, String) {
	let mut current_version: HKEY = MaybeUninit::uninit().assume_init();

	let status: u32 = RegOpenKeyExW(
		HKEY_LOCAL_MACHINE,
		"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"
			.encode_utf16()
			.chain(once(0))
			.collect::<Vec<u16>>()
			.as_ptr(),
		0,
		KEY_QUERY_VALUE,
		&mut current_version as *mut _,
	) as u32;
	if status != ERROR_SUCCESS {
		return ("<unknown>".to_owned(), "<unknown>".to_owned());
	}

	let version =
		get_string_value(current_version, buffer, "ProductName").unwrap_or("<unknown>".to_owned());
	let release =
		get_string_value(current_version, buffer, "BuildLabEx").unwrap_or("<unknown>".to_owned());

	RegCloseKey(current_version);

	(version, release)
}

unsafe fn get_cpu_arch() -> Arch {
	let mut sysinfo: SYSTEM_INFO = zeroed();
	GetSystemInfo(&mut sysinfo as *mut _);

	match sysinfo.u.s().wProcessorArchitecture {
		PROCESSOR_ARCHITECTURE_AMD64 => Arch::X86_64,
		PROCESSOR_ARCHITECTURE_ARM => Arch::ARM,
		PROCESSOR_ARCHITECTURE_ARM64 => Arch::AARCH64,
		PROCESSOR_ARCHITECTURE_INTEL => Arch::X86,
		PROCESSOR_ARCHITECTURE_UNKNOWN | PROCESSOR_ARCHITECTURE_IA64 | _ => Arch::Unknown,
	}
}

pub fn info() -> Info {
	let mut buffer: Vec<u16> = Vec::with_capacity(128);
	let mut hostname_len: u32 = buffer.capacity() as u32;
	unsafe { buffer.set_len(buffer.capacity()) };

	let hostname: String = match unsafe {
		GetComputerNameW(buffer.as_mut_ptr(), &mut hostname_len as *mut _)
	} {
		0 => "<unknown>".to_owned(),
		_ => String::from_utf16(&buffer[..hostname_len as usize]).unwrap_or("<unknown>".to_owned()),
	};

	let (version, release) = unsafe { get_version_and_release(&mut buffer) };

	Info {
		operating_system: OS::Windows,
		release,
		version,
		hostname,
		architecture: unsafe { get_cpu_arch() },
	}
}
