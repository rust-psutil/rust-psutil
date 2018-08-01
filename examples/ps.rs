//! Example psutil executable.

extern crate psutil;

#[cfg(not(test))]
fn main() {
    println!(
        "{:>5} {:^5} {:>8} {:>8} {:.100}",
        "PID", "STATE", "UTIME", "STIME", "CMD"
    );

    for p in &psutil::process::all().unwrap() {
        println!(
            "{:>5} {:^5} {:>8.2} {:>8.2} {:.100}",
            p.pid,
            p.state.to_string(),
            p.utime,
            p.stime,
            p.cmdline().unwrap().unwrap_or(format!("[{}]", p.comm))
        );
    }
}
