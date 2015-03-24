extern crate psutil;

use psutil::getpid;
use psutil::process::Process;

fn get_process() -> Process {
    Process::new(getpid()).unwrap()
}

#[test]
fn process() {
    assert!(Process::new(getpid()).is_ok());
}

#[test]
fn process_alive() {
    assert!(get_process().is_alive());
}

#[test]
fn process_cpu() {
    let process = get_process();
    assert!(process.utime  >= 0.0);
    assert!(process.stime  >= 0.0);
    assert!(process.cutime >= 0.0);
    assert!(process.cstime >= 0.0);
}

#[test]
fn process_cmdline() {
    assert!(get_process().cmdline().is_ok());
}

#[test]
fn process_memory() {
    assert!(get_process().memory().is_ok());
}
