use std::collections::HashMap;

use crate::process::{Process, ProcessError, ProcessResult};
use crate::{Count, Error, WindowsOsError};
use std::cmp::min;
use std::thread::sleep;
use std::time::Duration;

use winapi::shared::winerror::ERROR_PARTIAL_COPY;

use crate::process::ProcessDataKind;

pub struct IoCounters {}

pub trait ProcessExt {
	fn environ(&self) -> ProcessResult<HashMap<String, String>>;

	fn get_ionice(&self) -> i32;

	fn set_ionice(&self, nice: i32);

	fn io_counters(&self) -> IoCounters;

	fn num_handles(&self) -> Count;

	fn get_cpu_affinity(&self) -> i32;

	fn set_cpu_affinity(&self, nice: i32);

	fn memory_maps(&self);
}

impl ProcessExt for Process {
	fn environ(&self) -> ProcessResult<HashMap<String, String>> {
		let mut delay = 1;
		for _ in 0..33 {
			match unsafe { self.get_process_data(ProcessDataKind::ENVIRONMENT) } {
				Err(ProcessError::PsutilError {
					source:
						Error::WindowsError {
							source:
								WindowsOsError::Win32Error {
									code: ERROR_PARTIAL_COPY,
									..
								},
						},
					..
				}) => {
					sleep(Duration::from_millis(delay));
					delay = min(delay * 2, 40);
				}
				Err(e) => return Err(e),
				Ok(x) => {
					let mut terminator = x.len() - 1;
					let mut t: bool = false;
					for (i, x) in x.iter().enumerate() {
						if t && *x == 0 {
							terminator = i;
							break;
						} else if *x == 0 {
							t = true;
						} else {
							t = false;
						}
					}

					let mut map: HashMap<String, String> = HashMap::new();

					for x in x[..terminator].split(|x| *x == 0) {
						match String::from_utf16(x) {
							Ok(x) => {
								let (k, v) = process_environment_entry(&x);
								if !k.is_empty() {
									map.insert(k.to_string(), v.to_string());
								}
							}
							Err(e) => {
								return Err(ProcessError::PsutilError {
									pid: self.pid,
									source: Error::FromUtf16ConvertError { source: e },
								})
							}
						};
					}

					return Ok(map);
				}
			};
		}

		Err(ProcessError::AccessDenied { pid: self.pid })
	}

	fn get_ionice(&self) -> i32 {
		todo!()
	}

	fn set_ionice(&self, _nice: i32) {
		todo!()
	}

	fn io_counters(&self) -> IoCounters {
		todo!()
	}

	fn num_handles(&self) -> Count {
		todo!()
	}

	fn get_cpu_affinity(&self) -> i32 {
		todo!()
	}

	fn set_cpu_affinity(&self, _nice: i32) {
		todo!()
	}

	fn memory_maps(&self) {
		todo!()
	}
}

fn process_environment_entry(entry: &str) -> (&str, &str) {
	let delimiter = entry
		.chars()
		.enumerate()
		.find(|(i, x)| *x == '=' && *i != 0usize)
		.map(|(i, _)| i);

	let key = match delimiter {
		Some(d) => &entry[..d],
		None => &entry[..],
	};

	let value = match delimiter {
		Some(d) => &entry[d + 1..],
		None => "",
	};

	(key, value)
}

#[cfg(test)]
mod unit_tests {
	use super::*;

	#[test]
	fn test_process_environment() {
		assert_eq!(process_environment_entry("TEST=t1"), ("TEST", "t1"));
		assert_eq!(process_environment_entry("TEST="), ("TEST", ""));
		assert_eq!(process_environment_entry("TEST"), ("TEST", ""));
		assert_eq!(process_environment_entry("==TEST"), ("=", "TEST"));
		assert_eq!(process_environment_entry("===TEST"), ("=", "=TEST"));
		assert_eq!(process_environment_entry(""), ("", ""));

		assert!(Process::current().unwrap().environ().is_ok());
	}
}
