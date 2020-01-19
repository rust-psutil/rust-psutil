// https://github.com/heim-rs/heim/blob/master/heim-disk/src/sys/linux/counters.rs

use std::collections::HashMap;
use std::fs;
use std::io;
use std::str::FromStr;
use std::time::Duration;

use crate::utils::invalid_data;
use crate::{Bytes, Count};

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
            read_merged_count: try_parse!(fields[4]),
            write_merged_count: try_parse!(fields[8]),
            busy_time: Duration::from_millis(try_parse!(fields[12])),
        })
    }
}

/// Determine partitions we want to look for.
fn get_partitions(data: &str) -> io::Result<Vec<&str>> {
    let mut partitions: Vec<&str> = Vec::new();

    let lines: Vec<&str> = data.lines().collect();
    for line in lines.iter().skip(2) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() == 4 {
            partitions.push(fields[3]);
        } else {
            return Err(invalid_data(
                "failed to load partition information from '/proc/partitions'",
            ));
        }
    }

    Ok(partitions)
}

fn nowrap(prev: u64, current: u64, corrected: u64) -> u64 {
    if current >= prev {
        corrected + (current - prev)
    } else {
        corrected + current + ((std::u32::MAX as u64) - prev)
    }
}

fn fix_io_counter_overflow(
    prev: &HashMap<String, DiskIoCounters>,
    current: &HashMap<String, DiskIoCounters>,
    corrected: &HashMap<String, DiskIoCounters>,
) -> HashMap<String, DiskIoCounters> {
    let mut result: HashMap<String, DiskIoCounters> = HashMap::new();

    for (name, current_counters) in current {
        if !prev.contains_key(name) || !corrected.contains_key(name) {
            result.insert(name.clone(), current_counters.clone());
        } else {
            let prev_counters = &prev[name];
            let corrected_counters = &corrected[name];

            result.insert(
                name.clone(),
                DiskIoCounters {
                    read_count: nowrap(
                        prev_counters.read_count,
                        current_counters.read_count,
                        corrected_counters.read_count,
                    ),
                    write_count: nowrap(
                        prev_counters.write_count,
                        current_counters.write_count,
                        corrected_counters.write_count,
                    ),
                    read_bytes: nowrap(
                        prev_counters.read_bytes,
                        current_counters.read_bytes,
                        corrected_counters.read_bytes,
                    ),
                    write_bytes: nowrap(
                        prev_counters.write_bytes,
                        current_counters.write_bytes,
                        corrected_counters.write_bytes,
                    ),
                    read_time: Duration::from_millis(nowrap(
                        prev_counters.read_time.as_millis() as u64,
                        current_counters.read_time.as_millis() as u64,
                        corrected_counters.read_time.as_millis() as u64,
                    )),
                    write_time: Duration::from_millis(nowrap(
                        prev_counters.write_time.as_millis() as u64,
                        current_counters.write_time.as_millis() as u64,
                        corrected_counters.write_time.as_millis() as u64,
                    )),
                    read_merged_count: nowrap(
                        prev_counters.read_merged_count,
                        current_counters.read_merged_count,
                        corrected_counters.read_merged_count,
                    ),
                    write_merged_count: nowrap(
                        prev_counters.write_merged_count,
                        current_counters.write_merged_count,
                        corrected_counters.write_merged_count,
                    ),
                    busy_time: Duration::from_millis(nowrap(
                        prev_counters.busy_time.as_millis() as u64,
                        current_counters.busy_time.as_millis() as u64,
                        corrected_counters.busy_time.as_millis() as u64,
                    )),
                },
            );
        }
    }

    result
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
        let partitions = fs::read_to_string("/proc/partitions")?;
        let partitions = get_partitions(&partitions)?;

        let disk_stats = fs::read_to_string("/proc/diskstats")?;
        let lines: Vec<&str> = disk_stats.lines().collect();

        let mut io_counters: HashMap<String, DiskIoCounters> = HashMap::new();

        for line in lines {
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
