//! Contains functions to read and write pidfiles

use std::old_io::File;
use std::old_io::Writer;
use std::str::FromStr;

pub fn write_pidfile(path: &Path) {
    write!(&mut File::create(path).unwrap(), "{}", super::getpid()).unwrap();
}

pub fn read_pidfile(path: &Path) -> super::PID {
    let mut file = File::open(path).unwrap();
    let contents = file.read_to_string().unwrap();
    return FromStr::from_str(contents.as_slice()).unwrap();
}
