use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::disk::FileSystem;
use crate::utils::invalid_data;

#[derive(Clone, Debug)]
pub struct Partition {
    pub(crate) device: String,
    pub(crate) mountpoint: PathBuf,
    pub(crate) filesystem: FileSystem,
    pub(crate) mount_options: String,
}

impl Partition {
    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn mountpoint(&self) -> &Path {
        &self.mountpoint
    }

    /// Renamed from `fstype` in Python psutil.
    pub fn filesystem(&self) -> &FileSystem {
        &self.filesystem
    }

    /// Renamed from `opts` in Python psutil.
    pub fn mount_options(&self) -> &str {
        &self.mount_options
    }
}

/// Determine filesystem we want to look for
fn fstype(data: &str) -> Vec<&str> {
    let mut fstypes: Vec<&str> = Vec::new();

    let lines: Vec<&str> = data.lines().collect();
    for line in lines {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields[0] != "nodev" {
            fstypes.push(fields[0]);
        } else if fields[1] == "zfs" {
            fstypes.push(fields[1]);
        }
    }

    fstypes
}

fn partitions_internal(all: bool) -> io::Result<Vec<Partition>> {
    let filesystems = fs::read_to_string("/proc/filesystems")?;
    let fstypes = fstype(&filesystems);

    let mounts = fs::read_to_string("/proc/mounts")?;
    let mounts_lines: Vec<&str> = mounts.lines().collect();

    let mut partitions: Vec<Partition> = Vec::new();

    for line in mounts_lines {
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() < 4 {
            return Err(invalid_data(
                "failed to load partition information on '/proc/mounts'",
            ));
        }

        if all || fstypes.contains(&fields[2]) {
            partitions.push(Partition {
                device: String::from(fields[0]),
                mountpoint: PathBuf::from(fields[1]),
                filesystem: FileSystem::from_str(fields[2]).unwrap(),
                mount_options: String::from(fields[3]),
            });
        }
    }

    Ok(partitions)
}

pub fn partitions() -> io::Result<Vec<Partition>> {
    partitions_internal(true)
}

pub fn partitions_physical() -> io::Result<Vec<Partition>> {
    partitions_internal(false)
}
