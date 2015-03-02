//! Kill a process, reading it's PID from a file

#![feature(path)]

extern crate psutil;

use std::path::Path;
use psutil::process::Process;

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pidfile = Path::new(&args[1][..]);
    let process = Process::new_from_pidfile(&pidfile).unwrap();

    if let Err(error) = process.kill() {
        println!("Failed to kill process: {}.", error);
    };
}
