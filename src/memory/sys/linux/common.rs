use std::collections::HashMap;

use snafu::{ensure, ResultExt};

use crate::{MissingData, ParseInt, Result};

// TODO: should we only parse the ints that we need?
pub(crate) fn make_map<'a>(content: &'a str, path: &str) -> Result<HashMap<&'a str, u64>> {
	content
		.lines()
		.map(|line| {
			let fields: Vec<&str> = line.split_whitespace().collect();

			ensure!(
				fields.len() >= 2,
				MissingData {
					path,
					contents: line,
				}
			);

			let mut parsed = fields[1].parse().context(ParseInt {
				path,
				contents: line,
			})?;
			// only needed for `/proc/meminfo`
			if fields.len() >= 3 && fields[2] == "kB" {
				parsed *= 1024;
			}

			// only needed for `/proc/meminfo`
			let name = fields[0].trim_end_matches(':');

			Ok((name, parsed))
		})
		.collect()
}
