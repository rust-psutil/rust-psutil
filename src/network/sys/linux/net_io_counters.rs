use std::collections::HashMap;
use std::fs;
use std::io;
use std::str::FromStr;

use crate::network::NetIoCounters;
use crate::utils::invalid_data;

impl FromStr for NetIoCounters {
	type Err = std::io::Error;

	fn from_str(line: &str) -> Result<Self, Self::Err> {
		let fields: Vec<&str> = line.split_whitespace().collect();

		if fields.len() < 17 {
			return Err(invalid_data(
				"'/proc/net/dev' does not have the right number of fields",
			));
		}

		Ok(NetIoCounters {
			bytes_sent: try_parse!(fields[9]),
			bytes_recv: try_parse!(fields[1]),
			packets_sent: try_parse!(fields[10]),
			packets_recv: try_parse!(fields[2]),
			err_in: try_parse!(fields[3]),
			err_out: try_parse!(fields[11]),
			drop_in: try_parse!(fields[4]),
			drop_out: try_parse!(fields[12]),
		})
	}
}

pub(crate) fn net_io_counters_pernic() -> io::Result<HashMap<String, NetIoCounters>> {
	fs::read_to_string("/proc/net/dev")?
		.lines()
		.skip(2)
		.map(|line| {
			let fields: Vec<&str> = line.split_whitespace().collect();

			if fields.len() < 17 {
				return Err(invalid_data(
					"'/proc/net/dev' does not have the right number of fields",
				));
			}

			let mut net_name = String::from(fields[0]);
			// remove the trailing colon
			net_name.pop();

			Ok((net_name, NetIoCounters::from_str(&line)?))
		})
		.collect()
}
