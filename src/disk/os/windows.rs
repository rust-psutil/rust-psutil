use std::time::Duration;

use crate::disk::{DiskIoCounters, Partition};

pub trait DiskIoCountersExt {
	fn read_time(&self) -> Duration;

	fn write_time(&self) -> Duration;
}

impl DiskIoCountersExt for DiskIoCounters {
	fn read_time(&self) -> Duration {
		todo!()
	}

	fn write_time(&self) -> Duration {
		todo!()
	}
}

pub trait PartitionExt {
	fn name(&self) -> Option<&str>;
}

impl PartitionExt for Partition {
	fn name(&self) -> Option<&str> {
		if let Some(s) = self.name.as_ref() {
			Some(&s)
		} else {
			None
		}
	}
}
