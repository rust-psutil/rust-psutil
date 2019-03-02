//! Load disk informations
//! Author : Adrien Gaillard

extern crate libc;

use std::collections::HashMap;
use std::ffi::CString;
use std::io::{Error, ErrorKind, Result};
use std::mem;
use std::path::Path;
use utils::read_file;

/// Struct that contains informations about mounted partition
#[derive(Debug, Clone)]
pub struct MountedPartition {
    /// This field describes the block special device or remote filesystem to be mounted.
    pub device: String,

    /// This field describes the block special device or remote filesystem to be mounted.
    pub mountpoint: String,

    /// This field describes the type of the filesystem.
    pub fstype: String,

    /// This field describes the mount options associated with the filesystem.
    pub opts: String,
}

/// Struct that contains disk usage informations
#[derive(Copy, Clone, Debug)]
pub struct DiskUsage {
    /// Total disk in bytes
    pub total: u64,

    /// Disk used part in bytes
    pub used: u64,

    /// Disk free part in bytes
    pub free: u64,

    /// Percentage of used disk
    pub percent: f64,

    /// Number of inodes available
    pub disk_inodes_free: u64,

    /// Number of inodes for this filesystem
    pub disk_inodes_total: u64,

    /// Number of used inodes
    pub disk_inodes_used: u64,
}

/// Disk counter struct
#[derive(Clone, Copy, Debug)]
pub struct DiskIOCounters {
    /// Number of reads
    pub read_count: u64,

    /// Number of writes
    pub write_count: u64,

    /// Number of bytes read
    pub read_bytes: u64,

    /// Number of bytes written
    pub write_bytes: u64,

    /// Time spent reading from disk (in milliseconds)
    pub read_time: u64,

    /// Time spent writing to disk (in milliseconds)
    pub write_time: u64,

    /// Number of merged read
    pub read_merged_count: u64,

    /// Number of merged write
    pub write_merged_count: u64,

    /// Time spent doing actual I/Os (in milliseconds)
    pub busy_time: u64,
}

/// Disk counter struct to use nowrap mode
#[derive(Clone, Debug, Default)]
pub struct DiskIOCountersCollector {
    /// Save the total of counters
    disk_io_counters: HashMap<String, DiskIOCounters>,

    /// Save the values of the last call of disk_io_counters
    disk_io_counters_last_call: HashMap<String, DiskIOCounters>,
}

impl DiskIOCountersCollector {
    /// Reset de cache for disk_io_counter in nowrap mode
    pub fn cache_clear(&mut self) {
        self.disk_io_counters = HashMap::new();
        self.disk_io_counters_last_call = HashMap::new();
    }

    /// Return system-wide disk I/O statistics per disk as a vector of a DiskIOCounters structs
    ///
    /// If nowrap is true psutil will detect and adjust those numbers across
    /// function calls and add “old value” to “new value” so that the returned
    /// numbers will always be increasing or remain the same, but never decrease.
    /// <DiskIOCountersCollector>.cache_clear() can be used to invalidate the nowrap cache.
    pub fn disk_io_counters_perdisk(
        &mut self,
        nowrap: bool,
    ) -> Result<HashMap<String, DiskIOCounters>> {
        let partitions = read_file(Path::new("/proc/partitions"))?;
        let partitions = get_partitions(&partitions)?;
        let disk_stats = read_file(Path::new("/proc/diskstats"))?;
        let lines: Vec<&str> = disk_stats.lines().collect();
        let mut disks_infos: HashMap<String, DiskIOCounters> = HashMap::new();

        for line in lines {
            let mut disk_infos: Vec<&str> = line.split_whitespace().collect();
            // This function does not support kernel version under 2.6+.
            // These versions do not have 14 fields per line in /proc/diskstats.
            if disk_infos.len() == 14 {
                let name: &str = disk_infos[2];
                disk_infos.remove(2);
                disk_infos.remove(1);
                disk_infos.remove(0);
                let disk_infos: Vec<u64> = line_disk_stats(disk_infos)?;

                if partitions.contains(&name) {
                    let ssize = get_sector_size(name)?;
                    disks_infos.insert(
                        String::from(name),
                        DiskIOCounters {
                            read_count: disk_infos[0],
                            write_count: disk_infos[4],
                            read_bytes: disk_infos[2] * ssize,
                            write_bytes: disk_infos[6] * ssize,
                            read_time: disk_infos[3],
                            write_time: disk_infos[7],
                            read_merged_count: disk_infos[1],
                            write_merged_count: disk_infos[5],
                            busy_time: disk_infos[9],
                        },
                    );
                }
            } else {
                return Err(Error::new(
                      ErrorKind::InvalidData,
                      "/proc/diskstats has ne the right number of values. Maybe your kernel version is too old (Kernel 2.6+ minimum).".to_string(),
                  ));
            }
        }

        if nowrap {
            if !self.disk_io_counters.is_empty() {
                self.disk_io_counters = total_disk_io_counters(
                    &self.disk_io_counters_last_call,
                    &disks_infos,
                    &self.disk_io_counters,
                );
                self.disk_io_counters_last_call = disks_infos;
            } else {
                self.disk_io_counters = disks_infos.clone();
                self.disk_io_counters_last_call = disks_infos;
            }
            Ok(self.disk_io_counters.clone())
        } else {
            Ok(disks_infos)
        }
    }
}

/// Determine filesystem we want to look for
fn fstype(data: &str) -> Vec<&str> {
    let lines: Vec<&str> = data.lines().collect();
    let mut fstypes: Vec<&str> = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields[0] == "nodev" && fields[1] == "zfs" {
            fstypes.push(fields[1]);
        }
        if fields[0] != "nodev" {
            fstypes.push(fields[0]);
        }
    }
    fstypes
}

/// Determine partitions we want to look for
fn get_partitions(data: &str) -> Result<Vec<&str>> {
    let mut lines: Vec<&str> = data.lines().collect();
    // Removal of the two first line of /proc/partitions.
    // This two lines countains no usefull informations.
    if lines.len() >= 2 {
        lines.remove(1);
        lines.remove(0);
    }
    let mut partitions: Vec<&str> = Vec::new();
    for line in lines.iter() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() == 4 {
            partitions.push(fields[3]);
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "failed to load partition information on /proc/partitions".to_string(),
            ));
        }
    }
    Ok(partitions)
}

/// Determine the sector size of the partition name given in parameter
fn get_sector_size(partition_name: &str) -> Result<u64> {
    let path = format!("/sys/block/{}/queue/hw_sector_size", partition_name);
    let partition_size = match read_file(Path::new(&path)) {
        Ok(r) => r,
        // man iostat states that sectors are equivalent with blocks and
        // have a size of 512 bytes since 2.4 kernels
        Err(_) => return Ok(512),
    };
    match partition_size.trim().parse::<u64>() {
        Ok(v) => Ok(v),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("failed to parse {} in get_sector_size", partition_size),
        )),
    }
}

/// Convert a vector of &str in vector of u64. Values are from /proc/diskstats
fn line_disk_stats(line: Vec<&str>) -> Result<Vec<u64>> {
    let mut result: Vec<u64> = Vec::new();
    for value in line {
        result.push(match value.parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to parse {} in get_sector_size", value),
                ))
            }
        });
    }

    Ok(result)
}

/// Calculate
fn nowrap(past_value: u64, current_value: u64, total_value: u64) -> u64 {
    const MAX_VALUE: u64 = 4_294_967_296;
    if current_value >= past_value {
        total_value + current_value - past_value
    } else {
        total_value + current_value + MAX_VALUE - past_value
    }
}

/// Calculate per disk the new DiskIOCounters after a call of disk_io_counters_perdisk
fn total_disk_io_counters(
    past_disk_io_counters: &HashMap<String, DiskIOCounters>,
    current_disk_io_counters: &HashMap<String, DiskIOCounters>,
    total_disk_io_counters: &HashMap<String, DiskIOCounters>,
) -> HashMap<String, DiskIOCounters> {
    let mut final_disk_io_counters: HashMap<String, DiskIOCounters> = HashMap::new();
    for (name, current_counters) in current_disk_io_counters {
        if past_disk_io_counters.contains_key(name) && total_disk_io_counters.contains_key(name) {
            let past_counters = past_disk_io_counters[name];
            let total_counters = total_disk_io_counters[name];
            final_disk_io_counters.insert(
                name.clone(),
                DiskIOCounters {
                    read_count: nowrap(
                        past_counters.read_count,
                        current_counters.read_count,
                        total_counters.read_count,
                    ),
                    write_count: nowrap(
                        past_counters.write_count,
                        current_counters.write_count,
                        total_counters.write_count,
                    ),
                    read_bytes: nowrap(
                        past_counters.read_bytes,
                        current_counters.read_bytes,
                        total_counters.read_bytes,
                    ),
                    write_bytes: nowrap(
                        past_counters.write_bytes,
                        current_counters.write_bytes,
                        total_counters.write_bytes,
                    ),
                    read_time: nowrap(
                        past_counters.read_time,
                        current_counters.read_time,
                        total_counters.read_time,
                    ),
                    write_time: nowrap(
                        past_counters.write_time,
                        current_counters.write_time,
                        total_counters.write_time,
                    ),
                    read_merged_count: nowrap(
                        past_counters.read_merged_count,
                        current_counters.read_merged_count,
                        total_counters.read_merged_count,
                    ),
                    write_merged_count: nowrap(
                        past_counters.write_merged_count,
                        current_counters.write_merged_count,
                        total_counters.write_merged_count,
                    ),
                    busy_time: nowrap(
                        past_counters.busy_time,
                        current_counters.busy_time,
                        total_counters.busy_time,
                    ),
                },
            );
        } else {
            final_disk_io_counters.insert(name.clone(), *current_counters);
        }
    }
    final_disk_io_counters
}

/// Return all mounted disk partitions as a DiskPartitions struct including device,
/// mount point and filesystem type.
///
/// Similarly to “df” command on UNIX.
/// If all parameter is false it tries to distinguish and return physical devices only
/// (e.g. hard disks, cd-rom drives, USB keys) and ignore all others
/// (e.g. memory partitions such as /dev/shm).
pub fn disk_partitions(all: bool) -> Result<Vec<MountedPartition>> {
    let fstypes = read_file(Path::new("/proc/filesystems"))?;
    let fstypes = fstype(&fstypes);
    let partitions = read_file(Path::new("/proc/mounts"))?;
    let partitions_lines: Vec<&str> = partitions.lines().collect();
    let mut mounted_partitions: Vec<MountedPartition> = Vec::new();
    for line in partitions_lines {
        let partition_infos: Vec<&str> = line.split_whitespace().collect();
        if partition_infos.len() >= 4 && all {
            mounted_partitions.push(MountedPartition {
                device: String::from(partition_infos[0]),
                mountpoint: String::from(partition_infos[1]),
                fstype: String::from(partition_infos[2]),
                opts: String::from(partition_infos[3]),
            });
        }
        if partition_infos.len() >= 4
            && !all
            && partition_infos[0] != ""
            && fstypes.contains(&partition_infos[2])
        {
            mounted_partitions.push(MountedPartition {
                device: String::from(partition_infos[0]),
                mountpoint: String::from(partition_infos[1]),
                fstype: String::from(partition_infos[2]),
                opts: String::from(partition_infos[3]),
            });
        }
        if partition_infos.len() < 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "failed to load partition information on /proc/mounts".to_string(),
            ));
        }
    }
    Ok(mounted_partitions)
}

/// Return disk usage associated with path.
///
/// Note: UNIX usually reserves 5% disk space which is not accessible
/// by user. In this function "total" and "used" values reflect the
/// total and used disk space whereas "free" and "percent" represent
/// the "free" and "used percent" user disk space.
#[allow(unsafe_code)]
pub fn disk_usage(path: &str) -> Result<DiskUsage> {
    let mut buf: libc::statvfs = unsafe { mem::uninitialized() };
    let path = CString::new(path).unwrap();
    let result = unsafe { libc::statvfs(path.as_ptr(), &mut buf) };
    if result != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "failed to use statvfs : statvfs return an error code".to_string(),
        ));
    }
    let total = buf.f_blocks as u64 * buf.f_frsize;
    let avail_to_root = buf.f_bfree as u64 * buf.f_frsize;
    let free = buf.f_bavail as u64 * buf.f_frsize;
    let used = total - avail_to_root;
    let total_user = used + free;
    let percent = if total_user > 0 {
        used as f64 / total_user as f64 * 100.
    } else {
        0.
    };
    let disk_inodes_free = buf.f_ffree as u64;
    let disk_inodes_total = buf.f_files as u64;
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

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn line_disk_stats_test() {
        let entry: Vec<&str> = vec!["15", "8", "5652335", "4682", "645", "96"];
        let espected: Vec<u64> = vec![15, 8, 5652335, 4682, 645, 96];

        let result = match line_disk_stats(entry) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(result, espected);
    }

    #[test]
    fn get_partitions_test() {
        let entry: &str = "major minor  #blocks  name

   7        0      88684 loop0
   7        1      88980 loop1
   7        2      88964 loop2
   8        0  250059096 sda
   8        1     524288 sda1
   8        2     249023 sda2
   8        3  249283584 sda3
 253        0  249281536 dm-0
 253        1   73240576 dm-1
 253        2   97652736 dm-2
 253        3    7811072 dm-3
";
        let espected: Vec<&str> = vec![
            "loop0", "loop1", "loop2", "sda", "sda1", "sda2", "sda3", "dm-0", "dm-1", "dm-2",
            "dm-3",
        ];

        let result = match get_partitions(entry) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(result, espected);
    }

    #[test]
    fn fstype_test() {
        let entry: &str = "nodev	sysfs
nodev	rootfs
nodev	ramfs
nodev	bdev
nodev	proc
nodev	cpuset
nodev	cgroup
nodev	cgroup2
nodev	tmpfs
nodev	devtmpfs
nodev	configfs
nodev	debugfs
nodev	tracefs
nodev	securityfs
nodev	sockfs
nodev	dax
nodev	bpf
nodev	pipefs
nodev	hugetlbfs
nodev	devpts
	ext3
	ext2
	ext4
	squashfs
	vfat
nodev	ecryptfs
	fuseblk
nodev	fuse
nodev	fusectl
nodev	pstore
nodev	efivarfs
nodev   zfs
nodev	mqueue
	btrfs
nodev	autofs
nodev	overlay
";
        let espected: Vec<&str> = vec![
            "ext3", "ext2", "ext4", "squashfs", "vfat", "fuseblk", "zfs", "btrfs",
        ];

        let result = fstype(entry);
        assert_eq!(result, espected);
    }
}
