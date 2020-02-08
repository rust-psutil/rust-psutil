use std::io;

use crate::cpu::{cpu_times, cpu_times_percpu, CpuTimes};
use crate::utils::calculate_cpu_percent;
use crate::Percent;

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
	pub(crate) steal: Option<Percent>,
	#[cfg(target_os = "linux")]
	pub(crate) guest: Option<Percent>,
	#[cfg(target_os = "linux")]
	pub(crate) guest_nice: Option<Percent>,
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
	pub fn idle(&self) -> Percent {
		#[cfg(target_os = "linux")]
		{
			self.idle + self.iowait
		}
		#[cfg(target_os = "macos")]
		{
			self.idle
		}
	}

	/// New method, not in Python psutil.
	pub fn busy(&self) -> Percent {
		#[cfg(target_os = "linux")]
		{
			// https://github.com/giampaolo/psutil/blob/e65cc95de72828caed74c7916530dd74fca351e3/psutil/__init__.py#L1653
			// On Linux guest times are already accounted in "user" or
			// "nice" times.
			// Htop does the same. References:
			// https://github.com/giampaolo/psutil/pull/940
			// http://unix.stackexchange.com/questions/178045
			// https://github.com/torvalds/linux/blob/
			//     447976ef4fd09b1be88b316d1a81553f1aa7cd07/kernel/sched/
			//     cputime.c#L158
			self.user
				+ self.system + self.nice
				+ self.irq + self.softirq
				+ self.steal.unwrap_or_default()
		}
		#[cfg(target_os = "macos")]
		{
			self.user + self.system + self.nice
		}
	}
}

fn calculate_cpu_times_percent(first: &CpuTimes, second: &CpuTimes) -> CpuTimesPercent {
	let first_total = first.total();
	let second_total = second.total();

	// first_total can equal second_total if cpu_times_percent is called multiple times in succession
	// first_total can also be greater than second_total at least on Linux although idk why
	if first_total >= second_total {
		return CpuTimesPercent::default();
	}

	let total_diff = second_total - first_total;

	let user = calculate_cpu_percent(first.user, second.user, total_diff);
	let system = calculate_cpu_percent(first.system, second.system, total_diff);
	let idle = calculate_cpu_percent(first.idle, second.idle, total_diff);
	let nice = calculate_cpu_percent(first.nice, second.nice, total_diff);

	#[cfg(target_os = "linux")]
	let iowait = calculate_cpu_percent(first.iowait, second.iowait, total_diff);
	#[cfg(target_os = "linux")]
	let irq = calculate_cpu_percent(first.irq, second.irq, total_diff);
	#[cfg(target_os = "linux")]
	let softirq = calculate_cpu_percent(first.softirq, second.softirq, total_diff);

	#[cfg(target_os = "linux")]
	let steal = first.steal.and_then(|first| {
		second
			.steal
			.map(|second| calculate_cpu_percent(first, second, total_diff))
	});
	#[cfg(target_os = "linux")]
	let guest = first.guest.and_then(|first| {
		second
			.guest
			.map(|second| calculate_cpu_percent(first, second, total_diff))
	});
	#[cfg(target_os = "linux")]
	let guest_nice = first.guest_nice.and_then(|first| {
		second
			.guest_nice
			.map(|second| calculate_cpu_percent(first, second, total_diff))
	});

	CpuTimesPercent {
		user,
		system,
		idle,
		nice,

		#[cfg(target_os = "linux")]
		iowait,
		#[cfg(target_os = "linux")]
		irq,
		#[cfg(target_os = "linux")]
		softirq,
		#[cfg(target_os = "linux")]
		steal,
		#[cfg(target_os = "linux")]
		guest,
		#[cfg(target_os = "linux")]
		guest_nice,
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
