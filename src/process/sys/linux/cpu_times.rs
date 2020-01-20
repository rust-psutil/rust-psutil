use std::time::Duration;

use crate::process::os::linux::ProcfsStat;

#[derive(Clone, Debug)]
pub struct ProcessCpuTimes {
	pub(crate) user: Duration,
	pub(crate) system: Duration,
	pub(crate) children_user: Duration,
	pub(crate) children_system: Duration,
	pub(crate) iowait: Duration,
}

impl ProcessCpuTimes {
	pub fn user(&self) -> Duration {
		self.user
	}

	pub fn system(&self) -> Duration {
		self.system
	}

	pub fn children_user(&self) -> Duration {
		self.children_user
	}

	pub fn children_system(&self) -> Duration {
		self.children_system
	}

	/// New method, not in Python psutil.
	pub fn busy(&self) -> Duration {
		self.user() + self.system()
	}
}

impl From<ProcfsStat> for ProcessCpuTimes {
	fn from(stat: ProcfsStat) -> Self {
		ProcessCpuTimes {
			user: stat.utime,
			system: stat.stime,
			children_user: stat.cutime,
			children_system: stat.cstime,
			iowait: Duration::default(), // TODO
		}
	}
}
