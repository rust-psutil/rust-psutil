use std::collections::HashMap;
use std::fs;
use std::io;
use std::time::Duration;

use crate::process::{stat, statm, Process, ProcessCpuTimes, ProcessResult, Stat, StatM};
use crate::utils::invalid_data;

fn parse_environ(data: &str) -> io::Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    for mapping in data.split_terminator('\0') {
        let split: Vec<&str> = mapping.splitn(2, '=').collect();
        if split.len() != 2 {
            return Err(invalid_data(&format!(
                "malformed env mapping: '{}'",
                mapping
            )));
        }
        env.insert(split[0].to_owned(), split[1].to_owned());
    }

    Ok(env)
}

pub struct IoCounters {}

pub trait ProcessExt {
    fn environ(&self) -> io::Result<HashMap<String, String>>;

    fn get_ionice(&self) -> i32;

    fn set_ionice(&self, nice: i32);

    fn get_rlimit(&self) -> i32;

    fn set_rlimit(&self, nice: i32);

    fn io_counters(&self) -> IoCounters;

    fn get_cpu_affinity(&self) -> i32;

    fn set_cpu_affinity(&self, nice: i32);

    fn cpu_num(&self) -> i32;

    fn memory_maps(&self);

    /// New method, not in Python psutil
    fn stat(&self) -> ProcessResult<Stat>;

    /// New method, not in Python psutil
    fn statm(&self) -> ProcessResult<StatM>;
}

impl ProcessExt for Process {
    fn environ(&self) -> io::Result<HashMap<String, String>> {
        let data = fs::read_to_string(self.procfs_path("environ"))?;

        parse_environ(&data)
    }

    fn get_ionice(&self) -> i32 {
        todo!()
    }

    fn set_ionice(&self, _nice: i32) {
        todo!()
    }

    fn get_rlimit(&self) -> i32 {
        todo!()
    }

    fn set_rlimit(&self, _nice: i32) {
        todo!()
    }

    fn io_counters(&self) -> IoCounters {
        todo!()
    }

    fn get_cpu_affinity(&self) -> i32 {
        todo!()
    }

    fn set_cpu_affinity(&self, _nice: i32) {
        todo!()
    }

    fn cpu_num(&self) -> i32 {
        todo!()
    }

    fn memory_maps(&self) {
        todo!()
    }

    fn stat(&self) -> ProcessResult<Stat> {
        stat(self.pid)
    }

    fn statm(&self) -> ProcessResult<StatM> {
        statm(self.pid)
    }
}

pub trait ProcessCpuTimesExt {
    fn iowait(&self) -> Duration;
}

impl ProcessCpuTimesExt for ProcessCpuTimes {
    fn iowait(&self) -> Duration {
        self.iowait
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_parse_environ() {
        let data = "HOME=/\0init=/sbin/init\0recovery=\0TERM=linux\0BOOT_IMAGE=/boot/vmlinuz-3.13.0-128-generic\0PATH=/sbin:/usr/sbin:/bin:/usr/bin\0PWD=/\0rootmnt=/root\0";
        let env = parse_environ(data).unwrap();
        assert_eq!(env["HOME"], "/");
        assert_eq!(env["rootmnt"], "/root");
        assert_eq!(env["recovery"], "");
    }
}
