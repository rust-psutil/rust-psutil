//! Kill a process, reading it's PID from a file

#![feature(env)]
#![feature(old_path)]

extern crate psutil;

use psutil::process::Process;

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pidfile = Path::new(args[1].clone());
    let process = Process::new_from_pidfile(&pidfile).unwrap();

    if let Err(error) = process.kill() {
        println!("Failed to kill process: {}.", error);
    };
}
