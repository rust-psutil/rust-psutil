//! Example psutil executable.

extern crate psutil;

#[cfg(not(test))]
fn main() {
    println!("Processes: {}", psutil::process::all().len());
    println!("System uptime: {} seconds", psutil::system::uptime());
}
