#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Temperature;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "renamed_serde"))]
#[derive(Debug, Clone)]
pub struct TemperatureSensor {
	pub(crate) unit: String,
	pub(crate) label: Option<String>,
	pub(crate) current: Temperature,
	pub(crate) max: Option<Temperature>,
	pub(crate) crit: Option<Temperature>,
	pub(crate) min: Option<Temperature>,
	pub(crate) hwmon_id: Option<String>,
}

impl TemperatureSensor {
	/// Returns sensor unit name.
	pub fn unit(&self) -> &str {
		&self.unit
	}

	/// Returns sensor label.
	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	/// Returns current temperature reported by sensor.
	pub fn current(&self) -> &Temperature {
		&self.current
	}

	/// Returns high trip point for sensor if available.
	pub fn high(&self) -> Option<&Temperature> {
		self.max.as_ref()
	}

	/// Returns min trip point for sensor if available.
	pub fn min(&self) -> Option<&Temperature> {
		self.min.as_ref()
	}

	/// Returns critical trip point for sensor if available.
	pub fn critical(&self) -> Option<&Temperature> {
		self.crit.as_ref()
	}

	/// Returns the `hwmon_id` for the sensor if available.
	///
	/// Extracts the sensor ID from `/sys/class/hwmon/hwmon0` to identify the sensor.
	///
	/// Returns Some("hwmon0")
	pub fn hwmon_id(&self) -> Option<&str> {
		self.hwmon_id.as_deref()
	}
}
