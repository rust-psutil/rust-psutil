#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Mhz;

#[cfg_attr(feature = "serde", serde(crate = "renamed_serde"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CpuFreq {
	current: Mhz,
	min: Mhz,
	max: Mhz,
}

impl CpuFreq {
	pub fn new(current: Mhz, min: Mhz, max: Mhz) -> Self {
		CpuFreq { current, min, max }
	}
	pub fn current(&self) -> Mhz {
		self.current
	}
	pub fn min(&self) -> Mhz {
		self.min
	}
	pub fn max(&self) -> Mhz {
		self.max
	}
}
