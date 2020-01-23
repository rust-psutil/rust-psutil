use std::collections::HashMap;
use std::io;

use crate::disk::DiskIoCounters;

pub(crate) fn disk_io_counters_perdisk() -> io::Result<HashMap<String, DiskIoCounters>> {
	todo!()
}
