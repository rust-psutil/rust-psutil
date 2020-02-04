use std::io;
use std::time::Duration;

use crate::cpu::{cpu_times, cpu_times_percpu, CpuTimes};
use crate::utils::calculate_cpu_percent;
use crate::Percent;

#[cfg(target_os = "linux")]
use crate::cpu::os::linux::CpuTimesPercentExt as _;
#[cfg(target_family = "unix")]
use crate::cpu::os::unix::CpuTimesPercentExt as _;

/// Every attribute represents the percentage of time the CPU has spent in the given mode.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CpuTimesPercent {
	pub(crate) user: Percent,
	pub(crate) system: Percent,
	pub(crate) idle: Percent,
	pub(crate) nice: Percent,

	#[cfg(target_os = "linux")]
	pub(crate) iowait: Percent,
	#[cfg(target_os = "linux")]
	pub(crate) irq: Percent,
	#[cfg(target_os = "linux")]
	pub(crate) softirq: Percent,
	#[cfg(target_os = "linux")]
	pub(crate) steal: Percent,
	#[cfg(target_os = "linux")]
	pub(crate) guest: Percent,
	#[cfg(target_os = "linux")]
	pub(crate) guest_nice: Percent,
}

impl CpuTimesPercent {
	/// Time spent by normal processes executing in user mode;
	/// on Linux this also includes guest time.
	pub fn user(&self) -> Percent {
		self.user
	}

	/// Time spent by processes executing in kernel mode.
	pub fn system(&self) -> Percent {
		self.system
	}

	/// Time spent doing nothing.
	#[cfg(target_os = "linux")]
	pub fn idle(&self) -> Percent {
		self.idle + self.iowait()
	}

	/// Time spent doing nothing.
	#[cfg(target_os = "macos")]
	pub fn idle(&self) -> Percent {
		self.idle
	}

	/// New method, not in Python psutil.
	#[cfg(target_os = "linux")]
	pub fn busy(&self) -> Percent {
		self.user()
			+ self.system()
			+ self.nice()
			+ self.irq() + self.softirq()
			+ self.steal()
			+ self.guest()
			+ self.guest_nice()
	}

	/// New method, not in Python psutil.
	#[cfg(target_os = "macos")]
	pub fn busy(&self) -> Percent {
		self.user() + self.system() + self.nice()
	}
}

fn calculate_cpu_times_percent(first: &CpuTimes, second: &CpuTimes) -> CpuTimesPercent {
	let total_diff = second.total() - first.total();

	// total_diff can be 0 if cpu_times_percent is called multiple times in succession
	if total_diff == Duration::default() {
		return CpuTimesPercent::default();
	}

	CpuTimesPercent {
		user: calculate_cpu_percent(first.user, second.user, total_diff),
		system: calculate_cpu_percent(first.system, second.system, total_diff),
		idle: calculate_cpu_percent(first.idle, second.idle, total_diff),
		nice: calculate_cpu_percent(first.nice, second.nice, total_diff),
		#[cfg(target_os = "linux")]
		iowait: calculate_cpu_percent(first.iowait, second.iowait, total_diff),
		#[cfg(target_os = "linux")]
		irq: calculate_cpu_percent(first.irq, second.irq, total_diff),
		#[cfg(target_os = "linux")]
		softirq: calculate_cpu_percent(first.softirq, second.softirq, total_diff),
		#[cfg(target_os = "linux")]
		steal: calculate_cpu_percent(first.steal, second.steal, total_diff),
		#[cfg(target_os = "linux")]
		guest: calculate_cpu_percent(first.guest, second.guest, total_diff),
		#[cfg(target_os = "linux")]
		guest_nice: calculate_cpu_percent(first.guest_nice, second.guest_nice, total_diff),
	}
}

/// Get `CpuTimesPercent`s in non-blocking mode.
///
/// Example:
///
/// ```
/// let mut cpu_times_percent_collector = psutil::cpu::CpuTimesPercentCollector::new().unwrap();
///
/// let cpu_times_percent = cpu_times_percent_collector.cpu_times_percent().unwrap();
/// let cpu_times_percent_percpu = cpu_times_percent_collector.cpu_times_percent_percpu().unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct CpuTimesPercentCollector {
	cpu_times: CpuTimes,
	cpu_times_percpu: Vec<CpuTimes>,
}

impl CpuTimesPercentCollector {
	/// Initialize the `CpuTimesPercentCollector` so the method calls are ready to be used.
	pub fn new() -> io::Result<CpuTimesPercentCollector> {
		let cpu_times = cpu_times()?;
		let cpu_times_percpu = cpu_times_percpu()?;

		Ok(CpuTimesPercentCollector {
			cpu_times,
			cpu_times_percpu,
		})
	}

	/// Returns a `CpuTimesPercent` since the last time this was called or since
	/// `CpuTimesPercentCollector::new()` was called.
	pub fn cpu_times_percent(&mut self) -> io::Result<CpuTimesPercent> {
		let current_cpu_times = cpu_times()?;
		let cpu_percent_since = calculate_cpu_times_percent(&self.cpu_times, &current_cpu_times);
		self.cpu_times = current_cpu_times;

		Ok(cpu_percent_since)
	}

	/// Returns a `CpuTimesPercent` for each cpu since the last time this was called or since
	/// `CpuTimesPercentCollector::new()` was called.
	pub fn cpu_times_percent_percpu(&mut self) -> io::Result<Vec<CpuTimesPercent>> {
		let current_cpu_times_percpu = cpu_times_percpu()?;
		let vec = self
			.cpu_times_percpu
			.iter()
			.zip(current_cpu_times_percpu.iter())
			.map(|(prev, cur)| calculate_cpu_times_percent(prev, &cur))
			.collect();
		self.cpu_times_percpu = current_cpu_times_percpu;

		Ok(vec)
	}
}
