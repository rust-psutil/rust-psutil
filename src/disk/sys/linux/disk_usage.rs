use std::ffi::CString;
use std::io;
use std::mem;
use std::path::Path;

use crate::utils::invalid_data;
use crate::{Bytes, Percent};

#[derive(Clone, Debug)]
pub struct DiskUsage {
    pub(crate) total: Bytes,
    pub(crate) used: Bytes,
    pub(crate) free: Bytes,
    pub(crate) percent: Percent,

    // TODO: expose these in an OS extension
    /// Number of inodes available.
    pub(crate) disk_inodes_free: Bytes,

    // TODO: expose these in an OS extension
    /// Number of inodes for this filesystem.
    pub(crate) disk_inodes_total: Bytes,

    // TODO: expose these in an OS extension
    /// Number of used inodes.
    pub(crate) disk_inodes_used: Bytes,
}

impl DiskUsage {
    /// Total disk size in bytes.
    pub fn total(&self) -> Bytes {
        self.total
    }

    /// Number of bytes used.
    pub fn used(&self) -> Bytes {
        self.used
    }

    /// Number of bytes free.
    pub fn free(&self) -> Bytes {
        self.free
    }

    /// Percentage of disk used.
    pub fn percent(&self) -> Percent {
        self.percent
    }
}

pub fn disk_usage<P>(path: P) -> io::Result<DiskUsage>
where
    P: AsRef<Path>,
{
    let mut buf = mem::MaybeUninit::<libc::statvfs>::uninit();
    let path = CString::new(path.as_ref().to_string_lossy().to_string()).unwrap();
    let result = unsafe { libc::statvfs(path.as_ptr(), buf.as_mut_ptr()) };
    if result != 0 {
        return Err(invalid_data(
            "failed to use statvfs: statvfs return an error code",
        ));
    }
    let buf = unsafe { buf.assume_init() };

    let total = buf.f_blocks * buf.f_frsize;
    let avail_to_root = buf.f_bfree * buf.f_frsize;
    let free = buf.f_bavail * buf.f_frsize;
    let used = total - avail_to_root;
    let total_user = used + free;
    let percent = if total_user > 0 {
        (used as f64 / total_user as f64) * 100.0
    } else {
        0.0
    } as f32;
    let disk_inodes_free = buf.f_ffree;
    let disk_inodes_total = buf.f_files;
    let disk_inodes_used = if disk_inodes_total >= disk_inodes_free {
        disk_inodes_total - disk_inodes_free
    } else {
        100
    };

    Ok(DiskUsage {
        total,
        used,
        free,
        percent,
        disk_inodes_free,
        disk_inodes_total,
        disk_inodes_used,
    })
}
