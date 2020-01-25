use std::time::Duration;

use crate::process::ProcessCpuTimes;

pub trait ProcessCpuTimesExt {
	fn iowait(&self) -> Duration;
}

impl ProcessCpuTimesExt for ProcessCpuTimes {
	fn iowait(&self) -> Duration {
		self.iowait
	}
}
