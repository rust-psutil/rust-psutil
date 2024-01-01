// https://github.com/heim-rs/heim/blob/master/heim-process/src/sys/macos/process/cpu_times.rs
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::time::Duration;

#[cfg(target_os = "linux")]
use crate::process::os::linux::ProcfsStat;

#[cfg(target_os = "macos")]
pub(crate) static MACH_TIMEBASE_INFO: once_cell::sync::Lazy<mach::mach_time::mach_timebase_info> =
	once_cell::sync::Lazy::new(|| {
		let mut timebase_info = mach::mach_time::mach_timebase_info { numer: 0, denom: 0 };
		let timebase_info_result =
			unsafe { mach::mach_time::mach_timebase_info(&mut timebase_info) };
		if timebase_info_result != mach::kern_return::KERN_SUCCESS {
			panic!(
				"mach_timebase_info failed: {}",
				std::io::Error::last_os_error()
			)
		}
		timebase_info
	});

#[cfg_attr(feature = "serde", serde(crate = "renamed_serde"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ProcessCpuTimes {
	pub(crate) user: Duration,
	pub(crate) system: Duration,
	pub(crate) children_user: Duration,
	pub(crate) children_system: Duration,

	#[cfg(target_os = "linux")]
	pub(crate) iowait: Option<Duration>,
}

impl ProcessCpuTimes {
	pub fn user(&self) -> Duration {
		self.user
	}

	pub fn system(&self) -> Duration {
		self.system
	}

	pub fn children_user(&self) -> Duration {
		self.children_user
	}

	pub fn children_system(&self) -> Duration {
		self.children_system
	}

	/// New method, not in Python psutil.
	pub fn busy(&self) -> Duration {
		self.user() + self.system()
	}
}

#[cfg(target_os = "linux")]
impl From<&ProcfsStat> for ProcessCpuTimes {
	fn from(procfs_stat: &ProcfsStat) -> Self {
		ProcessCpuTimes {
			user: procfs_stat.utime,
			system: procfs_stat.stime,
			children_user: procfs_stat.cutime,
			children_system: procfs_stat.cstime,
			iowait: procfs_stat.delayacct_blkio,
		}
	}
}

#[cfg(target_os = "macos")]
impl From<darwin_libproc::proc_taskinfo> for ProcessCpuTimes {
	fn from(info: darwin_libproc::proc_taskinfo) -> Self {
		ProcessCpuTimes {
			user: Duration::from_nanos(
				(info.pti_total_user as u64 * MACH_TIMEBASE_INFO.numer as u64)
					/ MACH_TIMEBASE_INFO.denom as u64,
			),
			system: Duration::from_nanos(
				(info.pti_total_system as u64 * MACH_TIMEBASE_INFO.numer as u64)
					/ MACH_TIMEBASE_INFO.denom as u64,
			),
			children_user: Duration::default(),
			children_system: Duration::default(),
		}
	}
}
