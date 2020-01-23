use std::collections::HashMap;
use std::io;

use crate::disk::DiskIoCounters;

pub(crate) fn disk_io_counters_per_partition() -> io::Result<HashMap<String, DiskIoCounters>> {
	todo!()
}
