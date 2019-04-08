//! Contains functions to read and write pidfiles.

use std::fs::{self, File};
use std::io::Write;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str::FromStr;

/// Writes the PID of the current process to a file.
pub fn write_pidfile<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    return write!(&mut File::create(path)?, "{}", super::getpid());
}

/// Reads a PID from a file.
pub fn read_pidfile<P>(path: P) -> Result<super::PID>
where
    P: AsRef<Path>,
{
    let contents = fs::read_to_string(&path)?;

    match FromStr::from_str(&contents) {
        Ok(pid) => Ok(pid),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Could not parse pidfile as PID",
        )),
    }
}
