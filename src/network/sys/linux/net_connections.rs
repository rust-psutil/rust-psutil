use crate::common::TcpConnectionStatus;
use crate::{Fd, Pid};

pub struct NetConnection {}

impl NetConnection {
	pub fn fd(&self) -> Option<Fd> {
		todo!()
	}

	// TODO: return type
	pub fn family(&self) {
		todo!()
	}

	// TODO: return type
	pub fn address_type(&self) {
		todo!()
	}

	// TODO: return type
	pub fn local_addr(&self) {
		todo!()
	}

	// TODO: return type
	pub fn remote_addr(&self) {
		todo!()
	}

	pub fn status(&self) -> TcpConnectionStatus {
		todo!()
	}

	pub fn pid(&self) -> Option<Pid> {
		todo!()
	}
}

pub enum ConnectionKind {
	Inet,
	Inet4,
	Inet6,
	Tcp,
	Tcp4,
	Tcp6,
	Udp,
	Udp4,
	Udp6,
	Unix,
	All,
}

pub fn net_connections(_kind: ConnectionKind) -> Vec<NetConnection> {
	todo!()
}
