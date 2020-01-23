use std::time::SystemTime;

use crate::Pid;

pub struct User {}

impl User {
	pub fn user(&self) -> &str {
		todo!()
	}

	pub fn terminal(&self) -> Option<&str> {
		todo!()
	}

	pub fn host(&self) -> Option<&str> {
		todo!()
	}

	pub fn started(&self) -> SystemTime {
		todo!()
	}

	pub fn pid(&self) -> Option<Pid> {
		todo!()
	}
}
