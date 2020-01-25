use std::net::IpAddr;

pub struct NetIfAddr {}

impl NetIfAddr {
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
