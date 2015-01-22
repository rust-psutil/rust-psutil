//! Read information from the /proc filesystem

#![experimental]
#![allow(unstable)]

use std::io::fs;
use std::io::IoResult;
use std::io::fs::File;
use std::str::FromStr;
use std::str::StrExt;
use std::vec::Vec;

/// Int alias for process IDs
pub type PID = isize;

/// A process with a PID
#[derive(PartialEq,Eq,Copy,Show)]
pub struct Process {
    pub pid: PID
}

impl Process {
    /// Return a vector of all processes in /proc
    pub fn all() -> Vec<Process> {
        let mut processes = Vec::new();

        for path in fs::readdir(&Path::new("/proc")).unwrap().iter() {
            match FromStr::from_str(path.filename_str().unwrap()) {
                Some(pid) => { processes.push(Process { pid: pid }) },
                None      => ()
            }
        }

        return processes;
    }

    /// Returns the raw `cmdline` for a process
    ///
    /// Arguments in a raw cmdline are split by null terminators (`\0`)
    fn cmdline_raw(&self) -> IoResult<String> {
        let path = Path::new(format!("/proc/{}/cmdline", self.pid));
        return File::open(&path).read_to_string();
    }

    /// Return the cmdline for a given PID as a vector
    ///
    /// The cmdline string is split by null terminators, but this function
    /// replaces them with spaces. This might not be the best approach - in the
    /// future this should probably return a list.
    pub fn cmdline(&self) -> IoResult<Vec<String>> {
        let cmdline = self.cmdline_raw().unwrap();
        // Split terminator skips empty trailing substrings
        let split = cmdline.split_terminator(
            |&: c: char| c == '\0' || c == ' ');
        // `split` returns a vector of slices viewing `cmdline`, so they
        // get mapped to actuall strings before being returned as a vector.
        return Ok(split.map(|x| x.to_string()).collect());
    }

    /// Return the commandline for a given PID as a String
    pub fn cmdline_str(&self) -> IoResult<String> {
        return Ok(try!(self.cmdline_raw()).replace("\0", " "));
    }
}
