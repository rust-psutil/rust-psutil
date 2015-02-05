//! Contains functions to read and write pidfiles

use std::old_io::File;
use std::old_io::IoError;
use std::old_io::IoErrorKind;
use std::old_io::IoResult;
use std::old_io::Writer;
use std::str::FromStr;

pub fn write_pidfile(path: &Path) -> IoResult<()> {
    return write!(&mut File::create(path).unwrap(), "{}", super::getpid());
}

pub fn read_pidfile(path: &Path) -> IoResult<super::PID> {
    let mut file = try!(File::open(path));
    let contents = try!(file.read_to_string());

    return match FromStr::from_str(contents.as_slice()) {
        Ok(pid) => Ok(pid),
        Err(_)  => Err(IoError {
            kind: IoErrorKind::InvalidInput,
            desc: "Could not parse pidfile as PID",
            detail: Some(contents)
        })
    };
}
