use std::time::Duration;

#[cfg(target_os = "linux")]
use crate::cpu::os::linux::CpuTimesExt as _;
#[cfg(target_family = "unix")]
use crate::cpu::os::unix::CpuTimesExt as _;

/// Every attribute represents the seconds the CPU has spent in the given mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuTimes {
	pub(crate) user: Duration,
	pub(crate) system: Duration,
	pub(crate) idle: Duration,
	pub(crate) nice: Duration,

	#[cfg(target_os = "linux")]
	pub(crate) iowait: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) irq: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) softirq: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) steal: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) guest: Duration,
	#[cfg(target_os = "linux")]
	pub(crate) guest_nice: Duration,
}

impl CpuTimes {
	/// Time spent by normal processes executing in user mode;
	/// on Linux this also includes guest time.
	pub fn user(&self) -> Duration {
		self.user
	}

	/// Time spent by processes executing in kernel mode.
	pub fn system(&self) -> Duration {
		self.system
	}

	/// Time spent doing nothing.
	pub fn idle(&self) -> Duration {
		if cfg!(target_os = "linux") {
			self.idle + self.iowait()
		} else if cfg!(target_os = "macos") {
			self.idle
		} else {
			todo!()
		}
	}

	/// New method, not in Python psutil.
	pub fn busy(&self) -> Duration {
		if cfg!(target_os = "linux") {
			self.user()
				+ self.system() + self.nice()
				+ self.irq() + self.softirq()
				+ self.steal() + self.guest()
				+ self.guest_nice()
		} else if cfg!(target_os = "macos") {
			self.user() + self.system() + self.nice()
		} else {
			todo!()
		}
	}

	/// New method, not in Python psutil.
	pub fn total(&self) -> Duration {
		self.busy() + self.idle()
	}
}
