use std::collections::HashMap;
use std::io;

use crate::Bytes;

pub enum Duplex {
	Full,
	Half,
	Unknown,
}

pub struct NetIfStats {}

impl NetIfStats {
	pub fn is_up(&self) -> bool {
		todo!()
	}

	pub fn duplex(&self) -> Duplex {
		todo!()
	}

	pub fn speed(&self) -> Bytes {
		todo!()
	}

	pub fn mtu(&self) -> Bytes {
		todo!()
	}
}

pub fn net_if_stats() -> io::Result<HashMap<String, NetIfStats>> {
	todo!()
}
