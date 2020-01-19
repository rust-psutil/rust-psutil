use psutil::{host, process};

fn main() {
    println!("Process count: {}", process::pids().unwrap().len());
    println!(
        "System uptime: {} seconds",
        host::uptime().unwrap().as_secs()
    );
}
