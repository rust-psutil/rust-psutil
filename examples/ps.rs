//! Example psutil executable

#![feature(core)]

extern crate psutil;

#[cfg(not(test))]
fn main() {
    println!("{:>5} {}", "PID", "CMD");

    // Print all processes that are not zombies
    for process in psutil::process::all().iter() {
        // Limited to 100 chars becuase working out the term width is a butt
        println!("{:>5} {:.100}", process.pid, process.extended_name());
    }
}
