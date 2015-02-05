#![feature(io)]
#![feature(path)]

extern crate psutil;

use psutil::pidfile::{read_pidfile,write_pidfile};
use std::old_io::TempDir;

#[test]
fn read_write_pidfile() {
    // This will be removed automatically when dropped
    let tmp = TempDir::new("psutil").unwrap();

    let mut path = Path::new(tmp.path());
    path.push("test");
    path.set_extension("pid");
    println!("{:?}", path);

    // Write the pidfile to the temporary directory
    write_pidfile(&path);

    // Read the pidfile and check it against `getpid()`
    assert_eq!(read_pidfile(&path), psutil::getpid());
}
