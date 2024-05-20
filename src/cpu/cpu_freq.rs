#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{cpu::cpu_freq_percpu, Mhz, Result};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "renamed_serde"))]
#[derive(Debug, Clone, PartialEq)]
pub struct CpuFreq {
	pub(crate) current: Mhz,
	pub(crate) min: Option<Mhz>,
	pub(crate) max: Option<Mhz>,
}

impl CpuFreq {
	#[must_use]
	pub const fn current(&self) -> Mhz {
		self.current
	}

	#[must_use]
	pub const fn min(&self) -> Option<Mhz> {
		self.min
	}

	#[must_use]
	pub const fn max(&self) -> Option<Mhz> {
		self.max
	}
}

pub fn cpu_freq() -> Result<Option<CpuFreq>> {
	let percpu = cpu_freq_percpu()?;

	Ok(if percpu.is_empty() {
		None
	} else if percpu.len() == 1 {
		Some(percpu[0].clone())
	} else {
		let cpu_count = percpu.len() as f64;
		let mut current = 0.0;
		let mut min = Some(0.0);
		let mut max = Some(0.0);
		for cpu in percpu {
			current += cpu.current;
			// On Linux if /proc/cpuinfo is used min/max are set to None.
			min = min.and_then(|x| cpu.min.map(|y| x + y));
			max = max.and_then(|x| cpu.max.map(|y| x + y));
		}

		current /= cpu_count;
		min.map(|x| x / cpu_count);
		max.map(|x| x / cpu_count);

		Some(CpuFreq { current, min, max })
	})
}
