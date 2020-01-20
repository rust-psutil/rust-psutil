use std::collections::HashMap;
use std::io;
use std::net::IpAddr;

pub struct NetIfAddrs {}

impl NetIfAddrs {
	// TODO: return type
	pub fn family(&self) {
		todo!()
	}

	pub fn address(&self) -> IpAddr {
		todo!()
	}

	pub fn netmask(&self) -> Option<IpAddr> {
		todo!()
	}

	pub fn broadcast(&self) -> Option<IpAddr> {
		todo!()
	}

	pub fn ptp(&self) -> Option<IpAddr> {
		todo!()
	}
}

pub fn net_if_addrs() -> io::Result<HashMap<String, Vec<NetIfAddrs>>> {
	todo!()
}
