extern crate psutil;

fn get_process() -> psutil::process::Process {
    psutil::process::Process::new(psutil::getpid()).unwrap()
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
fn process_cwd() {
    assert!(get_process().cwd().is_ok());
}

#[test]
fn process_exe() {
    assert!(get_process().exe().is_ok());
}

#[test]
fn process_memory() {
    get_process().memory().unwrap();
}

#[test]
fn process_equality() {
    assert_eq!(get_process(), get_process());
}

/// This could fail if you run the tests as PID 1. Please don't do that.
#[test]
fn process_inequality() {
    assert!(get_process() != psutil::process::Process::new(1).unwrap());
}

#[test]
fn all() {
    psutil::process::all().unwrap();
}
