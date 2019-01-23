extern crate psutil;
extern crate tempfile;

use std::fs::File;
use std::io::Write;

use psutil::pidfile::{read_pidfile, write_pidfile};

#[test]
fn read_write_pidfile() {
    // This will be removed automatically when dropped
    let tempdir = tempfile::Builder::new().prefix("psutil-tests").tempdir().unwrap();
    let pidfile = tempdir.path().join("read_write_pidfile.pid");

    // Write the pidfile to the temporary directory
    write_pidfile(&pidfile).unwrap();

    // Read the pidfile and check it against `getpid()`
    assert_eq!(read_pidfile(&pidfile).unwrap(), psutil::getpid());
}

#[test]
#[should_panic]
fn read_invalid_pidfile() {
    let tempdir = tempfile::Builder::new().prefix("psutil-tests").tempdir().unwrap();
    let pidfile = tempdir.path().join("read_invalid_pidfile.pid");

    write!(&mut File::create(&pidfile).unwrap(), "{}", "beans").unwrap();
    read_pidfile(&pidfile).unwrap();
}
