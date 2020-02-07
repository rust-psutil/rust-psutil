use std::time::Duration;

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
		#[cfg(target_os = "linux")]
		{
			self.idle + self.iowait
		}
		#[cfg(target_os = "macos")]
		{
			self.idle
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			todo!()
		}
	}

	/// New method, not in Python psutil.
	pub fn busy(&self) -> Duration {
		#[cfg(target_os = "linux")]
		{
			self.user
				+ self.system + self.nice
				+ self.irq + self.softirq
				+ self.steal + self.guest
				+ self.guest_nice
		}
		#[cfg(target_os = "macos")]
		{
			self.user + self.system + self.nice
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			todo!()
		}
	}

	/// New method, not in Python psutil.
	pub fn total(&self) -> Duration {
		self.busy() + self.idle()
	}
}
