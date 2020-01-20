use std::collections::HashMap;
use std::io;

fn get_multiplier(fields: &[&str]) -> Option<u64> {
	if fields.len() == 3 {
		match fields[2] {
			"kB" => Some(1024),
			_ => None,
		}
	} else {
		None
	}
}

pub(crate) fn make_map(data: &str) -> io::Result<HashMap<&str, u64>> {
	let mut map = HashMap::new();

	let lines: Vec<&str> = data.lines().collect();
	for line in lines {
		let fields: Vec<&str> = line.split_whitespace().collect();
		let key = fields[0];
		let mut value = fields[1].parse::<u64>().map_err(|_| {
			io::Error::new(
				io::ErrorKind::InvalidData,
				format!("failed to parse {}", key),
			)
		})?;

		if let Some(multiplier) = get_multiplier(&fields) {
			value *= multiplier;
		}

		map.insert(key, value);
	}

	Ok(map)
}
