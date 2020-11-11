use nix::libc::{c_double, getloadavg};

use crate::{host::LoadAvg, Result};

pub fn loadavg() -> Result<LoadAvg> {
	let mut data: [c_double; 3] = [0.0, 0.0, 0.0];
	unsafe {
		getloadavg(data.as_mut_ptr(), 3);
	}

	Ok(LoadAvg {
		one: data[0],
		five: data[1],
		fifteen: data[2],
	})
}
