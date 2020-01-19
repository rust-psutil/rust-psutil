use std::fs;
use std::io;

use crate::memory::make_map;
use crate::utils::not_found;
use crate::{Bytes, Percent};

#[derive(Debug, Clone)]
pub struct VirtualMemory {
    pub(crate) total: Bytes,
    pub(crate) available: Bytes,
    pub(crate) percent: Percent,
    pub(crate) used: Bytes,
    pub(crate) free: Bytes,
    pub(crate) active: Bytes,
    pub(crate) inactive: Bytes,
    pub(crate) buffers: Bytes,
    pub(crate) cached: Bytes,
    pub(crate) shared: Bytes,
}

impl VirtualMemory {
    /// Amount of total memory.
    pub fn total(&self) -> Bytes {
        self.total
    }

    /// Amount of memory available for new processes.
    pub fn available(&self) -> Bytes {
        self.available
    }

    /// Memory currently in use.
    pub fn used(&self) -> Bytes {
        self.used
    }

    /// Memory not being used.
    pub fn free(&self) -> Bytes {
        self.free
    }

    /// New method, not in Python psutil.
    /// Percent of memory used.
    pub fn percent(&self) -> Percent {
        self.percent
    }
}

pub fn virtual_memory() -> io::Result<VirtualMemory> {
    let data = fs::read_to_string("/proc/meminfo")?;
    let mem_info = make_map(&data)?;

    let total = *mem_info
        .get("MemTotal:")
        .ok_or_else(|| not_found("MemTotal"))?;
    let free = *mem_info
        .get("MemFree:")
        .ok_or_else(|| not_found("MemFree"))?;
    let buffers = *mem_info
        .get("Buffers:")
        .ok_or_else(|| not_found("Buffers"))?;
    let cached = *mem_info.get("Cached:").ok_or_else(|| not_found("Cached"))?
        // "free" cmdline utility sums reclaimable to cached.
        // Older versions of procps used to add slab memory instead.
        // This got changed in:
        //  https://gitlab.com/procps-ng/procps/commit/05d751c4f076a2f0118b914c5e51cfbb4762ad8e
        + *mem_info
            .get("SReclaimable:")
            .ok_or_else(|| not_found("SReclaimable"))?; // since kernel 2.6.19
    let active = *mem_info.get("Active:").ok_or_else(|| not_found("Active"))?;
    let inactive = *mem_info
        .get("Inactive:")
        .ok_or_else(|| not_found("Inactive"))?;

    // MemAvailable was introduced in kernel 3.14. The original psutil computes it if it's not
    // found, but since 3.14 has already reached EOL, let's assume that it's there.
    let available = *mem_info
        .get("MemAvailable:")
        .ok_or_else(|| not_found("MemAvailable"))?;

    // Shmem was introduced in 2.6.19
    let shared = *mem_info.get("Shmem:").ok_or_else(|| not_found("Shmem"))?;

    let used = total - free - cached - buffers;
    let percent = (((total as f64 - available as f64) / total as f64) * 100.0) as f32;

    Ok(VirtualMemory {
        total,
        available,
        shared,
        free,
        buffers,
        cached,
        active,
        inactive,
        used,
        percent,
    })
}
