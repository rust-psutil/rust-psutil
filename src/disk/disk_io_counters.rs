use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crate::disk::disk_io_counters_perdisk;
use crate::{Bytes, Count};

#[derive(Clone, Debug)]
pub struct DiskIoCounters {
	pub(crate) read_count: Count,
	pub(crate) write_count: Count,
	pub(crate) read_bytes: Bytes,
	pub(crate) write_bytes: Bytes,
	pub(crate) read_time: Duration,
	pub(crate) write_time: Duration,
	pub(crate) read_merged_count: Count,
	pub(crate) write_merged_count: Count,
	pub(crate) busy_time: Duration,
}

impl DiskIoCounters {
	/// Number of reads.
	pub fn read_count(&self) -> Count {
		self.read_count
	}

	/// Number of writes.
	pub fn write_count(&self) -> Count {
		self.write_count
	}

	/// Number of bytes read.
	pub fn read_bytes(&self) -> Bytes {
		self.read_bytes
	}

	/// Number of bytes written.
	pub fn write_bytes(&self) -> Bytes {
		self.write_bytes
	}
}

fn nowrap(prev: u64, current: u64, corrected: u64) -> u64 {
	if current >= prev {
		corrected + (current - prev)
	} else {
		corrected + current + ((std::u32::MAX as u64) - prev)
	}
}

fn nowrap_struct(
	prev: &DiskIoCounters,
	current: &DiskIoCounters,
	corrected: &DiskIoCounters,
) -> DiskIoCounters {
	DiskIoCounters {
		read_count: nowrap(prev.read_count, current.read_count, corrected.read_count),
		write_count: nowrap(prev.write_count, current.write_count, corrected.write_count),
		read_bytes: nowrap(prev.read_bytes, current.read_bytes, corrected.read_bytes),
		write_bytes: nowrap(prev.write_bytes, current.write_bytes, corrected.write_bytes),
		read_time: Duration::from_millis(nowrap(
			prev.read_time.as_millis() as u64,
			current.read_time.as_millis() as u64,
			corrected.read_time.as_millis() as u64,
		)),
		write_time: Duration::from_millis(nowrap(
			prev.write_time.as_millis() as u64,
			current.write_time.as_millis() as u64,
			corrected.write_time.as_millis() as u64,
		)),
		read_merged_count: nowrap(
			prev.read_merged_count,
			current.read_merged_count,
			corrected.read_merged_count,
		),
		write_merged_count: nowrap(
			prev.write_merged_count,
			current.write_merged_count,
			corrected.write_merged_count,
		),
		busy_time: Duration::from_millis(nowrap(
			prev.busy_time.as_millis() as u64,
			current.busy_time.as_millis() as u64,
			corrected.busy_time.as_millis() as u64,
		)),
	}
}

fn fix_io_counter_overflow(
	prev: &HashMap<String, DiskIoCounters>,
	current: &HashMap<String, DiskIoCounters>,
	corrected: &HashMap<String, DiskIoCounters>,
) -> HashMap<String, DiskIoCounters> {
	current
		.iter()
		.map(|(name, current_counters)| {
			if !prev.contains_key(name) || !corrected.contains_key(name) {
				(name.clone(), current_counters.clone())
			} else {
				let prev_counters = &prev[name];
				let corrected_counters = &corrected[name];

				(
					name.clone(),
					nowrap_struct(prev_counters, current_counters, corrected_counters),
				)
			}
		})
		.collect()
}

/// Used to persist data between calls to detect data overflow by the kernel and fix the result.
/// Requires a minimum kernel version of 2.5.69 due to the usage of `/proc/diskstats`.
#[derive(Clone, Debug, Default)]
pub struct DiskIoCountersCollector {
	prev_disk_io_counters_perdisk: Option<HashMap<String, DiskIoCounters>>,
	corrected_disk_io_counters_perdisk: Option<HashMap<String, DiskIoCounters>>,
}

impl DiskIoCountersCollector {
	pub fn disk_io_counters(&mut self) -> io::Result<DiskIoCounters> {
		todo!()
	}

	pub fn disk_io_counters_perdisk(&mut self) -> io::Result<HashMap<String, DiskIoCounters>> {
		let io_counters = disk_io_counters_perdisk()?;

		let corrected_counters = match (
			&self.prev_disk_io_counters_perdisk,
			&self.corrected_disk_io_counters_perdisk,
		) {
			(Some(prev), Some(corrected)) => {
				fix_io_counter_overflow(&prev, &io_counters, &corrected)
			}
			_ => io_counters.clone(),
		};

		self.prev_disk_io_counters_perdisk = Some(io_counters);
		self.corrected_disk_io_counters_perdisk = Some(corrected_counters.clone());

		Ok(corrected_counters)
	}
}
