use std::time::Duration;

#[cfg(target_os = "linux")]
use crate::process::os::linux::ProcfsStat;

#[derive(Clone, Debug)]
pub struct ProcessCpuTimes {
	pub(crate) user: Duration,
	pub(crate) system: Duration,
	pub(crate) children_user: Duration,
	pub(crate) children_system: Duration,

	#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
impl From<ProcfsStat> for ProcessCpuTimes {
	fn from(procfs_stat: ProcfsStat) -> Self {
		ProcessCpuTimes {
			user: procfs_stat.utime,
			system: procfs_stat.stime,
			children_user: procfs_stat.cutime,
			children_system: procfs_stat.cstime,
			iowait: Duration::default(), // TODO
		}
	}
}
