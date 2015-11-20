//! Example psutil executable.

extern crate psutil;

#[cfg(not(test))]
fn main() {
    println!("Processes: {}", psutil::process::all().unwrap().len());
    println!("System uptime: {} seconds", psutil::system::uptime());
}
