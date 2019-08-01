use std::io;
use std::time::Duration;

use crate::cpu::os::{linux::CpuTimesPercentExt as _, unix::CpuTimesPercentExt as _};
use crate::cpu::{cpu_times, cpu_times_percpu, CpuTimes};
use crate::Percent;

/// Every attribute represents the percentage of time the CPU has spent in the given mode.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuTimesPercent {
    pub(crate) user: Percent,
    pub(crate) nice: Percent,
    pub(crate) system: Percent,
    pub(crate) idle: Percent,
    pub(crate) iowait: Percent,
    pub(crate) irq: Percent,
    pub(crate) softirq: Percent,
    pub(crate) steal: Percent,
    pub(crate) guest: Percent,
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
    pub fn idle(&self) -> Percent {
        self.idle
    }

    /// New method, not in Python psutil.
    pub fn busy(&self) -> Percent {
        // TODO: what about guest and guest_nice?
        self.user()
            + self.system()
            + self.nice()
            + self.iowait() // TODO: is iowait idle time?
            + self.irq()
            + self.softirq()
            + self.steal()
    }
}

// TODO: fix casting
// TODO: use nightly div_duration_f32
#[allow(clippy::unnecessary_cast)]
fn delta_percentage(first: Duration, second: Duration, total_diff: Duration) -> Percent {
    (((second - first).as_nanos() as f64 / total_diff.as_nanos() as f64) * 100.0) as f32
}

fn calculate_cpu_times_percent(first: &CpuTimes, second: &CpuTimes) -> CpuTimesPercent {
    let total_diff = second.total() - first.total();

    CpuTimesPercent {
        user: delta_percentage(first.user, second.user, total_diff),
        nice: delta_percentage(first.nice, second.nice, total_diff),
        system: delta_percentage(first.system, second.system, total_diff),
        idle: delta_percentage(first.idle, second.idle, total_diff),
        iowait: delta_percentage(first.iowait, second.iowait, total_diff),
        irq: delta_percentage(first.irq, second.irq, total_diff),
        softirq: delta_percentage(first.softirq, second.softirq, total_diff),
        steal: delta_percentage(first.steal, second.steal, total_diff),
        guest: delta_percentage(first.guest, second.guest, total_diff),
        guest_nice: delta_percentage(first.guest_nice, second.guest_nice, total_diff),
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
