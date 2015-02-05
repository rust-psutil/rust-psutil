#![feature(io)]
#![feature(os)]
#![feature(path)]

extern crate psutil;

use std::old_io::File;

use psutil::pidfile::{read_pidfile,write_pidfile};

struct TempPidfile {
    path: Path
}

impl TempPidfile {
    fn new(name: &str) -> Self {
        let mut path = Path::new(std::os::tmpdir());
        path.push(name);
        path.set_extension("pid");
        return TempPidfile { path: path };
    }
}

impl Drop for TempPidfile {
    fn drop(&mut self) {
        std::old_io::fs::unlink(&self.path).unwrap();
    }
}

#[test]
fn read_write_pidfile() {
    // This will be removed automatically when dropped
    let pidfile = TempPidfile::new("read_write_pidfile");

    // Write the pidfile to the temporary directory
    write_pidfile(&pidfile.path).unwrap();

    // Read the pidfile and check it against `getpid()`
    assert_eq!(read_pidfile(&pidfile.path).unwrap(), psutil::getpid());
}

#[test]
fn read_invalid_pidfile() {
    let pidfile = TempPidfile::new("read_invalid_pidfile");

    write!(&mut File::create(&pidfile.path).unwrap(), "{}", "beans").unwrap();
    assert!(read_pidfile(&pidfile.path).is_err());
}
