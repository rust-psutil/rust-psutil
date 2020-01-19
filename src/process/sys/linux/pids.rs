use std::fs;
use std::io;
use std::path::Path;

use crate::Pid;

pub fn pids() -> io::Result<Vec<Pid>> {
    let mut pids = Vec::new();

    for entry in fs::read_dir("/proc")? {
        let filename = entry?.file_name();
        if let Ok(pid) = filename.to_string_lossy().parse::<Pid>() {
            pids.push(pid);
        }
    }

    Ok(pids)
}

pub fn pid_exists(pid: Pid) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}
