// https://github.com/heim-rs/heim/blob/master/heim-sensors/src/temperatures.rs
// https://github.com/heim-rs/heim/blob/master/heim-sensors/src/sys/linux/temperatures.rs

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use glob::glob;

use crate::sensors::TemperatureSensor;
use crate::utils::invalid_data;
use crate::Temperature;

#[inline]
fn file_name(prefix: &OsStr, postfix: &[u8]) -> OsString {
	let mut name = OsString::with_capacity(prefix.len() + postfix.len());
	name.push(prefix);
	name.push(OsStr::from_bytes(postfix));

	name
}

fn read_temperature(path: PathBuf) -> io::Result<Temperature> {
	let contents = fs::read_to_string(path)?;

	match contents.trim_end().parse::<f64>() {
		// Originally value is in millidegrees of Celsius
		Ok(value) => Ok(Temperature::new(value / 1_000.0)),
		Err(_e) => Err(invalid_data("Could not parse temperature")),
	}
}

fn hwmon_sensor(input: PathBuf) -> io::Result<TemperatureSensor> {
	// It is guaranteed by `hwmon` and `hwmon_sensor` directory traversals,
	// that it is not a root directory and it points to a file.
	// Otherwise it is an implementation bug.
	let root = input.parent().unwrap_or_else(|| unreachable!());
	let prefix = match input.file_name() {
		Some(name) => {
			let offset = name.len() - b"input".len();
			OsStr::from_bytes(&name.as_bytes()[..offset])
		}
		None => unreachable!(),
	};

	let unit = fs::read_to_string(root.join("name")).map(|mut string| {
		// Dropping trailing `\n`
		let _ = string.pop();
		string
	})?;

	let label_path = root.join(file_name(prefix, b"label"));
	let label = if label_path.exists() {
		fs::read_to_string(label_path).map(|mut string| {
			// Dropping trailing `\n`
			let _ = string.pop();
			Some(string)
		})?
	} else {
		None
	};

	let max_path = root.join(file_name(prefix, b"max"));
	let max = if max_path.exists() {
		read_temperature(max_path).map(Some)?
	} else {
		None
	};

	let crit_path = root.join(file_name(prefix, b"crit"));
	let crit = if crit_path.exists() {
		read_temperature(crit_path).map(Some)?
	} else {
		None
	};

	let current = read_temperature(input)?;

	Ok(TemperatureSensor {
		unit,
		label,
		current,
		max,
		crit,
	})
}

// https://github.com/shirou/gopsutil/blob/2cbc9195c892b304060269ef280375236d2fcac9/host/host_linux.go#L624
fn hwmon() -> Vec<io::Result<TemperatureSensor>> {
	let mut glob_results = glob("/sys/class/hwmon/hwmon*/temp*_input")
		.unwrap()
		.peekable(); // only errors on invalid pattern

	// checks if iterator is empty
	if glob_results.peek().is_none() {
		// CentOS has an intermediate /device directory:
		// https://github.com/giampaolo/psutil/issues/971
		// https://github.com/nicolargo/glances/issues/1060
		glob_results = glob("/sys/class/hwmon/hwmon*/device/temp*_input")
			.unwrap()
			.peekable();
	}

	glob_results
		.map(|result| match result {
			Ok(path) => hwmon_sensor(path),
			Err(e) => Err(e.into_error()),
		})
		.collect()
}

// https://www.kernel.org/doc/Documentation/thermal/sysfs-api.txt
fn thermal_zone() -> Vec<io::Result<TemperatureSensor>> {
	todo!()
}

pub fn temperatures() -> Vec<io::Result<TemperatureSensor>> {
	let hwmon = hwmon();

	if hwmon.is_empty() {
		thermal_zone()
	} else {
		hwmon
	}
}
