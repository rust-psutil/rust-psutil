//! Contains functions to read and write pidfiles.

use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

/// Writes the PID of the current process to a file.
pub fn write_pidfile(path: &Path) -> Result<()> {
    return write!(&mut File::create(path).unwrap(), "{}", super::getpid());
}

/// Reads a PID from a file.
pub fn read_pidfile(path: &Path) -> Result<super::PID> {
    let mut file = try!(File::open(path));
    let mut contents = String::new();
    try!(file.read_to_string(&mut contents));

    match FromStr::from_str(&contents) {
        Ok(pid) => Ok(pid),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Could not parse pidfile as PID",
        )),
    }
}
