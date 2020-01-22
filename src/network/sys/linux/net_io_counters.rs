use std::collections::HashMap;
use std::fs;
use std::io;

use crate::network::NetIoCounters;
use crate::utils::invalid_data;

pub(crate) fn net_io_counters_pernic() -> io::Result<HashMap<String, NetIoCounters>> {
	let net_dev = fs::read_to_string("/proc/net/dev")?;
	let net_lines: Vec<&str> = net_dev.lines().collect();

	let mut io_counters = HashMap::new();

	for line in net_lines.iter().skip(2) {
		let fields: Vec<&str> = line.split_whitespace().collect();

		if fields.len() < 17 {
			return Err(invalid_data(
				"'/proc/net/dev' does not have the right number of fields",
			));
		}

		let mut net_name = String::from(fields[0]);
		// remove the trailing colon
		net_name.pop();

		io_counters.insert(
			net_name,
			NetIoCounters {
				bytes_sent: try_parse!(fields[9]),
				bytes_recv: try_parse!(fields[1]),
				packets_sent: try_parse!(fields[10]),
				packets_recv: try_parse!(fields[2]),
				err_in: try_parse!(fields[3]),
				err_out: try_parse!(fields[11]),
				drop_in: try_parse!(fields[4]),
				drop_out: try_parse!(fields[12]),
			},
		);
	}

	Ok(io_counters)
}
