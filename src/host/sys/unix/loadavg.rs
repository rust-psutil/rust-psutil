use nix::libc::{c_double, getloadavg};

use crate::{host::LoadAvg, Error, Result};

pub fn loadavg() -> Result<LoadAvg> {
	let mut data: [c_double; 3] = [0.0, 0.0, 0.0];

	if unsafe { getloadavg(data.as_mut_ptr(), 3) } == -1 {
		return Err(Error::IRError { content: -1 });
	}

	Ok(LoadAvg {
		one: data[0],
		five: data[1],
		fifteen: data[2],
	})
}
