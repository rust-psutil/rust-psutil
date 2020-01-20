use std::collections::HashMap;
use std::fs;
use std::io;
use std::ops::Add;

use crate::utils::invalid_data;
use crate::{Bytes, Count};

#[derive(Clone, Debug, Default)]
pub struct NetIoCounters {
	pub(crate) bytes_sent: Bytes,
	pub(crate) bytes_recv: Bytes,
	pub(crate) packets_sent: Count,
	pub(crate) packets_recv: Count,
	pub(crate) err_in: Count,
	pub(crate) err_out: Count,
	pub(crate) drop_in: Count,
	pub(crate) drop_out: Count,
}

impl NetIoCounters {
	/// Number of bytes sent.
	pub fn bytes_sent(&self) -> Bytes {
		self.bytes_sent
	}

	/// Number of bytes received.
	pub fn bytes_recv(&self) -> Bytes {
		self.bytes_recv
	}

	/// Number of packets sent.
	pub fn packets_sent(&self) -> Count {
		self.packets_sent
	}

	/// Number of packets received.
	pub fn packets_recv(&self) -> Count {
		self.packets_recv
	}

	/// Total number of errors while receiving.
	pub fn err_in(&self) -> Count {
		self.err_in
	}

	/// Total number of errors while sending.
	pub fn err_out(&self) -> Count {
		self.err_out
	}

	/// Total number of incoming packets which were dropped.
	pub fn drop_in(&self) -> Count {
		self.drop_in
	}

	/// Total number of outgoing packets which were dropped (always 0 on macOS and BSD).
	pub fn drop_out(&self) -> Count {
		self.drop_out
	}
}

impl Add for NetIoCounters {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			bytes_sent: self.bytes_sent + other.bytes_sent,
			bytes_recv: self.bytes_recv + other.bytes_recv,
			packets_sent: self.packets_sent + other.packets_sent,
			packets_recv: self.packets_recv + other.packets_recv,
			err_in: self.err_in + other.err_in,
			err_out: self.err_out + other.err_out,
			drop_in: self.drop_in + other.drop_in,
			drop_out: self.drop_out + other.drop_out,
		}
	}
}

fn nowrap(prev: u64, current: u64, corrected: u64) -> u64 {
	if current >= prev {
		corrected + (current - prev)
	} else {
		corrected + current + ((std::u32::MAX as u64) - prev)
	}
}

fn fix_io_counter_overflow(
	prev: &HashMap<String, NetIoCounters>,
	current: &HashMap<String, NetIoCounters>,
	corrected: &HashMap<String, NetIoCounters>,
) -> HashMap<String, NetIoCounters> {
	let mut result: HashMap<String, NetIoCounters> = HashMap::new();

	for (name, current_counters) in current {
		if !prev.contains_key(name) || !corrected.contains_key(name) {
			result.insert(name.clone(), current_counters.clone());
		} else {
			let prev_counters = &prev[name];
			let corrected_counters = &corrected[name];

			result.insert(
				name.clone(),
				NetIoCounters {
					bytes_sent: nowrap(
						prev_counters.bytes_sent,
						current_counters.bytes_sent,
						corrected_counters.bytes_sent,
					),
					bytes_recv: nowrap(
						prev_counters.bytes_recv,
						current_counters.bytes_recv,
						corrected_counters.bytes_recv,
					),
					packets_sent: nowrap(
						prev_counters.packets_sent,
						current_counters.packets_sent,
						corrected_counters.packets_sent,
					),
					packets_recv: nowrap(
						prev_counters.packets_recv,
						current_counters.packets_recv,
						corrected_counters.packets_recv,
					),
					err_in: nowrap(
						prev_counters.err_in,
						current_counters.err_in,
						corrected_counters.err_in,
					),
					err_out: nowrap(
						prev_counters.err_out,
						current_counters.err_out,
						corrected_counters.err_out,
					),
					drop_in: nowrap(
						prev_counters.drop_in,
						current_counters.drop_in,
						corrected_counters.drop_in,
					),
					drop_out: nowrap(
						prev_counters.drop_out,
						current_counters.drop_out,
						corrected_counters.drop_out,
					),
				},
			);
		}
	}

	result
}

/// Used to persist data between calls to detect data overflow by the kernel and fix the result.
#[derive(Debug, Clone, Default)]
pub struct NetIoCountersCollector {
	prev_net_io_counters_pernic: Option<HashMap<String, NetIoCounters>>,
	corrected_net_io_counters_pernic: Option<HashMap<String, NetIoCounters>>,
}

impl NetIoCountersCollector {
	pub fn net_io_counters(&mut self) -> io::Result<NetIoCounters> {
		let sum = self
			.net_io_counters_pernic()?
			.into_iter()
			.map(|(_key, val)| val)
			.fold(NetIoCounters::default(), |start, item| start + item);

		Ok(sum)
	}

	pub fn net_io_counters_pernic(&mut self) -> io::Result<HashMap<String, NetIoCounters>> {
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

		let corrected_counters = match (
			&self.prev_net_io_counters_pernic,
			&self.corrected_net_io_counters_pernic,
		) {
			(Some(prev), Some(corrected)) => {
				fix_io_counter_overflow(&prev, &io_counters, &corrected)
			}
			_ => io_counters.clone(),
		};

		self.prev_net_io_counters_pernic = Some(io_counters);
		self.corrected_net_io_counters_pernic = Some(corrected_counters.clone());

		Ok(corrected_counters)
	}
}
