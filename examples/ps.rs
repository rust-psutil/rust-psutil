//! Example psutil executable

extern crate psutil;

#[cfg(not(test))]
fn main() {
    println!("{:>5} {}", "PID", "CMD");

    for process in psutil::process::all().iter().filter(|p| p.alive()) {
        println!("{:>5} {}", process.pid, process.cmdline_str().unwrap());
    }
}
