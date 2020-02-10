use std::str::FromStr;

use snafu::{ensure, ResultExt};

use crate::host::LoadAvg;
use crate::{read_file, Error, MissingData, ParseFloat, Result};

const PROC_LOADAVG: &str = "/proc/loadavg";

impl FromStr for LoadAvg {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		let fields: Vec<&str> = s.split_whitespace().collect();

		ensure!(
			fields.len() >= 3,
			MissingData {
				path: PROC_LOADAVG,
				contents: s,
			}
		);

		let parse = |s: &str| -> Result<f64> {
			s.parse().context(ParseFloat {
				path: PROC_LOADAVG,
				contents: s,
			})
		};

		let one = parse(fields[0])?;
		let five = parse(fields[1])?;
		let fifteen = parse(fields[2])?;

		Ok(LoadAvg { one, five, fifteen })
	}
}

pub fn loadavg() -> Result<LoadAvg> {
	LoadAvg::from_str(&read_file(PROC_LOADAVG)?)
}

#[cfg(test)]
mod unit_tests {
	use super::*;
	use crate::FloatCount;
	use float_cmp::approx_eq;

	#[test]
	fn test_loadaverage() {
		let loadavg = loadavg().unwrap();
		// shouldn't be negative
		assert!(loadavg.one >= 0.0);
		assert!(loadavg.five >= 0.0);
		assert!(loadavg.fifteen >= 0.0);
	}

	#[test]
	fn test_parse_loadavg() {
		let input = "0.49 0.70 0.84 2/519 1454\n";
		let loadavg = LoadAvg::from_str(input).unwrap();
		assert!(approx_eq!(FloatCount, loadavg.one, 0.49));
		assert!(approx_eq!(FloatCount, loadavg.five, 0.70));
		assert!(approx_eq!(FloatCount, loadavg.fifteen, 0.84));
	}
}
