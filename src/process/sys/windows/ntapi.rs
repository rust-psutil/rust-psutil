#![allow(non_snake_case)]

use winapi::shared::ntdef::UNICODE_STRING;
#[cfg(target_pointer_width = "32")]
use winapi::shared::{
	minwindef::BYTE,
	ntdef::{PVOID, PVOID64, USHORT},
};
use winapi::um::winnt::LPCWSTR;

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub struct UNICODE_STRING32 {
	pub Length: u16,
	pub MaxLength: u16,
	pub Buffer: u32,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS32 {
	pub Reserved1: [u8; 16],
	pub Reserved2: [u32; 5],
	pub CurrentDirectoryPath: UNICODE_STRING32,
	pub CurrentDirectoryHandle: u32,
	pub DllPath: UNICODE_STRING32,
	pub ImagePathName: UNICODE_STRING32,
	pub CommandLine: UNICODE_STRING32,
	pub env: u32,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub struct PEB32 {
	pub Reserved1: [u8; 2],
	pub BeingDebugged: u8,
	pub Reserved2: [u8; 1],
	pub Reserved3: [u32; 2],
	pub Ldr: u32,
	pub ProcessParameters: u32,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub struct PEB_ {
	pub Reserved1: [u8; 2],
	pub BeingDebugged: u8,
	pub Reserved2: [u8; 21],
	pub LoaderData: u64,
	pub ProcessParameters: *const RTL_USER_PROCESS_PARAMETERS_,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub struct PEB_ {
	pub Reserved1: [u8; 2],
	pub BeingDebugged: u8,
	pub Reserved2: [u8; 1],
	pub Reserved3: [u32; 2],
	pub Ldr: u32,
	pub ProcessParameters: *const RTL_USER_PROCESS_PARAMETERS_,
}

#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS_ {
	pub Reserved1: [u8; 16],
	pub Reserved2: [*const u8; 5],
	pub CurrentDirectoryPath: UNICODE_STRING,
	pub CurrentDirectoryHandle: *const u8,
	pub DllPath: UNICODE_STRING,
	pub ImagePathName: UNICODE_STRING,
	pub CommandLine: UNICODE_STRING,
	pub env: LPCWSTR,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub struct PROCESS_BASIC_INFORMATION64 {
	pub Reserved1: [PVOID; 2],
	pub PebBaseAddress: PVOID64,
	pub Reserved2: [PVOID; 4],
	pub UniqueProcessId: [PVOID; 2],
	pub Reserved3: [PVOID; 2],
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub struct PEB64 {
	pub Reserved1: [BYTE; 2],
	pub BeingDebugged: BYTE,
	pub Reserved2: [BYTE; 21],
	pub LoaderData: PVOID64,
	pub ProcessParameters: PVOID64,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Debug)]
pub struct UNICODE_STRING64 {
	pub Length: USHORT,
	pub MaxLength: USHORT,
	pub Buffer: PVOID64,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Debug)]
pub struct RTL_USER_PROCESS_PARAMETERS64 {
	pub Reserved1: [BYTE; 16],
	pub Reserved2: [PVOID64; 5],
	pub CurrentDirectoryPath: UNICODE_STRING64,
	pub CurrentDirectoryHandle: PVOID64,
	pub DllPath: UNICODE_STRING64,
	pub ImagePathName: UNICODE_STRING64,
	pub CommandLine: UNICODE_STRING64,
	pub env: PVOID64,
}
