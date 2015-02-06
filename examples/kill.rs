//! Kill a process, reading it's PID from a file

#![feature(os)]
#![feature(path)]

extern crate psutil;

use std::os;

use psutil::process::Process;

#[cfg(not(test))]
fn main() {
    let pidfile = Path::new(os::args()[1].clone());
    let process = Process::new_from_pidfile(&pidfile).unwrap();

    process.kill().unwrap();
}
