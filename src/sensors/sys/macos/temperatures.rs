use std::io;

use crate::Temperature;

#[derive(Debug)]
pub struct TemperatureSensor {
	unit: String,
	label: Option<String>,
	current: Temperature,
	max: Option<Temperature>,
	crit: Option<Temperature>,
}

impl TemperatureSensor {
	/// Returns sensor unit name.
	pub fn unit(&self) -> &str {
		&self.unit
	}

	/// Returns sensor label.
	pub fn label(&self) -> Option<&str> {
		self.label.as_ref().map(|s| s.as_str())
	}

	/// Returns current temperature reported by sensor.
	pub fn current(&self) -> &Temperature {
		&self.current
	}

	/// Returns high trip point for sensor if available.
	pub fn high(&self) -> Option<&Temperature> {
		self.max.as_ref()
	}

	/// Returns critical trip point for sensor if available.
	pub fn critical(&self) -> Option<&Temperature> {
		self.crit.as_ref()
	}
}

pub fn temperatures() -> Vec<io::Result<TemperatureSensor>> {
	Vec::new()
}
