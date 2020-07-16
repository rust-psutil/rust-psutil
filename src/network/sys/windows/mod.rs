use crate::network::NetIoCounters;
use crate::{Error, Result, WindowsOsError};
use std::collections::HashMap;
use std::{mem, ptr};

use winapi::shared::{
	netioapi::{GetIfEntry2, MIB_IF_ROW2},
	ntdef::ULONG,
	winerror::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS},
	ws2def::AF_UNSPEC,
};
use winapi::um::iphlpapi::GetAdaptersAddresses;
use winapi::um::iptypes::IP_ADAPTER_ADDRESSES;

unsafe fn get_nic_addresses() -> Result<Vec<IP_ADAPTER_ADDRESSES>> {
	let mut buffer_len_bytes: ULONG = 0;
	match GetAdaptersAddresses(
		AF_UNSPEC as u32,
		0,
		ptr::null_mut(),
		ptr::null_mut(),
		&mut buffer_len_bytes as *mut _,
	) {
		ERROR_BUFFER_OVERFLOW => (),
		e => {
			return Err(Error::from(WindowsOsError::from_code(
				e,
				"GetAdaptersAddresses",
			)))
		}
	}

	let buffer_len = {
		let mut x = buffer_len_bytes / mem::size_of::<IP_ADAPTER_ADDRESSES>() as u32;
		if buffer_len_bytes % mem::size_of::<IP_ADAPTER_ADDRESSES>() as u32 != 0 {
			x += 1;
		}
		x
	};

	// use IP_ADAPTER_ADDRESSES instead of u8 to force proper alignment
	let mut buffer: Vec<IP_ADAPTER_ADDRESSES> = vec![mem::zeroed(); buffer_len as usize];

	match GetAdaptersAddresses(
		AF_UNSPEC as u32,
		0,
		ptr::null_mut(),
		buffer.as_mut_ptr(),
		&mut buffer_len_bytes as *mut _,
	) {
		ERROR_SUCCESS => (),
		e => {
			return Err(Error::from(WindowsOsError::from_code(
				e,
				"GetAdaptersAddresses",
			)))
		}
	};

	return Ok(buffer);
}

pub(crate) fn net_io_counters_pernic() -> Result<HashMap<String, NetIoCounters>> {
	let mut last_error: Option<Error> = None;
	let mut map: HashMap<String, NetIoCounters> = HashMap::new();

	let nics = unsafe { get_nic_addresses()? };
	let mut nic_ptr: *const IP_ADAPTER_ADDRESSES = nics.as_ptr();

	while nic_ptr != ptr::null() {
		unsafe {
			let mut mir: MIB_IF_ROW2 = mem::MaybeUninit::uninit().assume_init();
			mir.InterfaceLuid = mem::zeroed();
			mir.InterfaceIndex = (*(nic_ptr)).u.s().IfIndex;

			match GetIfEntry2(&mut mir as *mut _) {
				ERROR_SUCCESS => {
					map.insert(
						mir.InterfaceIndex.to_string(),
						NetIoCounters {
							bytes_sent: (mir.OutOctets as u64).checked_mul(8).unwrap_or(u64::MAX),
							bytes_recv: (mir.InOctets as u64).checked_mul(8).unwrap_or(u64::MAX),
							packets_sent: mir.OutUcastPkts as u64 + mir.OutNUcastPkts as u64,
							packets_recv: mir.InUcastPkts as u64 + mir.InNUcastPkts as u64,
							err_in: mir.InErrors,
							err_out: mir.OutErrors,
							drop_in: mir.InDiscards,
							drop_out: mir.OutDiscards,
						},
					);
				}
				e => last_error = Some(Error::from(WindowsOsError::from_code(e, "GetIfEntry2"))),
			};

			nic_ptr = (*(nic_ptr)).Next;
		}
	}

	if let Some(e) = last_error {
		if map.is_empty() {
			return Err(e);
		}
	}

	Ok(map)
}
