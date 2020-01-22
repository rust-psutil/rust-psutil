use std::collections::HashMap;
use std::io;

use crate::Rpm;

pub struct FanSensor {
	_label: String,
	_current: Rpm,
}

pub fn fans() -> io::Result<HashMap<String, Vec<FanSensor>>> {
	todo!()
}
