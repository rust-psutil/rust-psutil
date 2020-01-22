// https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/linux/counters.rs

use std::collections::HashMap;
use std::fs;
use std::io;
use std::str::FromStr;
use std::time::Duration;

use crate::disk::DiskIoCounters;
use crate::utils::invalid_data;

// Copied from the `psutil` sources:
//
// "man iostat" states that sectors are equivalent with blocks and have
// a size of 512 bytes. Despite this value can be queried at runtime
// via /sys/block/{DISK}/queue/hw_sector_size and results may vary
// between 1k, 2k, or 4k... 512 appears to be a magic constant used
// throughout Linux source code:
// * https://stackoverflow.com/a/38136179/376587
// * https://lists.gt.net/linux/kernel/2241060
// * https://github.com/giampaolo/psutil/issues/1305
// * https://github.com/torvalds/linux/blob/4f671fe2f9523a1ea206f63fe60a7c7b3a56d5c7/include/linux/bio.h#L99
// * https://lkml.org/lkml/2015/8/17/234
const DISK_SECTOR_SIZE: u64 = 512;

impl FromStr for DiskIoCounters {
	type Err = io::Error;

	// At the moment supports format used in Linux 2.6+,
	// except ignoring discard values introduced in Linux 4.18.
	//
	// https://www.kernel.org/doc/Documentation/iostats.txt
	// https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats
	fn from_str(line: &str) -> io::Result<DiskIoCounters> {
		let fields: Vec<&str> = line.split_whitespace().collect();
		if fields.len() < 14 {
			return Err(invalid_data(
				"'/proc/diskstats' does not have the right number of values",
			));
		}
		Ok(DiskIoCounters {
			read_count: try_parse!(fields[3]),
			write_count: try_parse!(fields[7]),
			read_bytes: try_parse!(fields[5], u64::from_str) * DISK_SECTOR_SIZE,
			write_bytes: try_parse!(fields[9], u64::from_str) * DISK_SECTOR_SIZE,
			read_time: Duration::from_millis(try_parse!(fields[6])),
			write_time: Duration::from_millis(try_parse!(fields[10])),
			busy_time: Duration::from_millis(try_parse!(fields[12])),
			read_merged_count: try_parse!(fields[4]),
			write_merged_count: try_parse!(fields[8]),
		})
	}
}

/// Determine partitions we want to look for.
fn get_partitions(data: &str) -> io::Result<Vec<&str>> {
	data.lines()
		.skip(2)
		.map(|line| {
			let fields: Vec<&str> = line.split_whitespace().collect();
			if fields.len() != 4 {
				return Err(invalid_data(
					"failed to load partition information from '/proc/partitions'",
				));
			}

			Ok(fields[3])
		})
		.collect()
}

pub(crate) fn disk_io_counters_perdisk() -> io::Result<HashMap<String, DiskIoCounters>> {
	let data = fs::read_to_string("/proc/partitions")?;
	let partitions = get_partitions(&data)?;

	let disk_stats = fs::read_to_string("/proc/diskstats")?;

	let mut io_counters: HashMap<String, DiskIoCounters> = HashMap::new();

	for line in disk_stats.lines() {
		let fields: Vec<&str> = line.split_whitespace().collect();
		if fields.len() < 14 {
			return Err(invalid_data(
				"'/proc/diskstats' does not have the right number of values",
			));
		}
		let name = fields[2];
		if partitions.contains(&name) {
			io_counters.insert(String::from(name), DiskIoCounters::from_str(line)?);
		}
	}

	Ok(io_counters)
}
