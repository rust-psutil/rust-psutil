use std::collections::HashMap;
use std::io;

use crate::utils::invalid_data;

pub(crate) fn make_map(data: &str) -> io::Result<HashMap<&str, u64>> {
	data.lines()
		.map(|line| {
			let fields: Vec<&str> = line.split_whitespace().collect();
			if fields.len() < 2 {
				return Err(invalid_data(&format!(
					"Expected at least 2 fields, got {}",
					fields.len()
				)));
			}
			let mut value = try_parse!(fields[1]);
			if fields.len() == 3 && fields[2] == "kB" {
				value *= 1024;
			}

			Ok((fields[0], value))
		})
		.collect()
}
