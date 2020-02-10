use std::collections::HashMap;
use std::str::FromStr;

use snafu::{ensure, ResultExt};

use crate::network::NetIoCounters;
use crate::{read_file, Error, MissingData, ParseInt, Result};

const PROC_NET_DEV: &str = "/proc/net/dev";

impl FromStr for NetIoCounters {
	type Err = Error;

	fn from_str(line: &str) -> Result<Self> {
		let fields: Vec<&str> = line.split_whitespace().collect();

		ensure!(
			fields.len() >= 17,
			MissingData {
				path: PROC_NET_DEV,
				contents: line,
			}
		);

		let parse = |s: &str| -> Result<u64> {
			s.parse().context(ParseInt {
				path: PROC_NET_DEV,
				contents: line,
			})
		};

		Ok(NetIoCounters {
			bytes_sent: parse(fields[9])?,
			bytes_recv: parse(fields[1])?,
			packets_sent: parse(fields[10])?,
			packets_recv: parse(fields[2])?,
			err_in: parse(fields[3])?,
			err_out: parse(fields[11])?,
			drop_in: parse(fields[4])?,
			drop_out: parse(fields[12])?,
		})
	}
}

pub(crate) fn net_io_counters_pernic() -> Result<HashMap<String, NetIoCounters>> {
	read_file(PROC_NET_DEV)?
		.lines()
		.skip(2)
		.map(|line| {
			let fields: Vec<&str> = line.split_whitespace().collect();

			ensure!(
				fields.len() >= 17,
				MissingData {
					path: PROC_NET_DEV,
					contents: line,
				}
			);

			let mut net_name = String::from(fields[0]);
			// remove the trailing colon
			net_name.pop();

			Ok((net_name, NetIoCounters::from_str(&line)?))
		})
		.collect()
}
