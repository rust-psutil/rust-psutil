//! Read information from the /proc filesystem

#![allow(unstable)]

use std::io::fs;
use std::str::FromStr;
use std::io::IoResult;
use std::io::fs::File;
use std::io::fs::PathExtensions;

/// Int alias for process IDs
pub type PID = isize;

/// Return a list of each PID listed in /proc
pub fn pids() -> Vec<PID> {
    let mut pids: Vec<PID> = Vec::new();

    // Assume any directory that has a numeric ID is a process
    for path in fs::readdir(&Path::new("/proc")).unwrap().iter() {
        match FromStr::from_str(path.filename_str().unwrap()) {
            Some(n) => { assert!(path.is_dir()); pids.push(n) },
            None    => ()
        }
    }

    return pids;
}

pub fn cmdline(pid: PID) -> IoResult<String> {
    let path = Path::new(format!("/proc/{}/cmdline", pid));
    return File::open(&path).read_to_string();
}

