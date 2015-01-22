//! Example psutil executable

extern crate psutil;

#[cfg(not(test))]
fn main() {
    for process in psutil::process::all().iter() {
        let cmdline = process.cmdline().unwrap();
        if cmdline.len() > 0 {
            println!("{} $ {}", process.pid, cmdline[0]);
        }
    }
}
