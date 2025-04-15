#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Rpm;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "renamed_serde"))]
pub struct FanSensor {
	pub(crate) _label: String,
	pub(crate) _current: Rpm,
}
