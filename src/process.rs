//! Information about specific processes

use std::io::fs;
use std::io::File;
use std::io::IoError;
use std::io::IoErrorKind;
use std::io::IoResult;
use std::str::FromStr;
use std::str::StrExt;
use std::vec::Vec;

/// Int alias for process IDs
pub type PID = isize;

/// Possible statuses for a process
#[derive(Copy,Show)]
pub enum Status {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Traced,
    Paging
}

impl Status {
    /// Returns a Status based on a status character from `/proc/[pid]/stat`
    ///
    /// > One character from the string "RSDZTW" where R is running, S is
    /// > sleeping in an interruptible wait, D is waiting in uninterruptible
    /// > disk sleep, Z is zombie, T is traced or stopped (on a signal), and W
    /// > is paging.
    fn from_char(status: char) -> Status {
        match status {
            'R' => Status::Running,
            'S' => Status::Sleeping,
            'D' => Status::Waiting,
            'Z' => Status::Zombie,
            'T' => Status::Traced,
            'W' => Status::Paging,
            s => unreachable!("Invalid status character {}", s)
        }
    }
}

/// A process with a PID
#[derive(Show)]
pub struct Process {
    pub pid: PID,
    pub name: String,
    pub status: Status
}

impl Process {
    fn _read_proc(pid: PID, name: &str) -> IoResult<String> {
        let path = Path::new(format!("/proc/{}/{}", pid, name));
        return File::open(&path).read_to_string();
    }

    fn read_proc(&self, name: &str) -> IoResult<String> {
        return Process::_read_proc(self.pid, name);
    }

    /// Attempts to read process information from `/proc/[pid]/stat`.
    pub fn new(pid: PID) -> IoResult<Process> {
        let contents = try!(Process::_read_proc(pid, "stat"));
        let stat: Vec<&str> = contents.split(' ').collect();

        // This may only be the case for Linux, but this can be removed or
        // changed when/if support for other kernels is needed
        assert!(stat.len() == 52, "Unknown stat fields");

        // Read each field into an attribute for a new Process instance
        return Ok(Process {
            pid: FromStr::from_str(stat[0]).unwrap(),
            name: {
                let name = stat[1].to_string();
                name[1..name.len()-1].to_string()
            },
            status: Status::from_char(stat[2].chars().next().unwrap())
        });
    }

    /// Return `true` if the process is/was alive (at the time it was read).
    #[experimental]
    pub fn alive(&self) -> bool {
        match self.status {
            Status::Zombie => false,
            _ => true
        }
    }

    fn cmdline_raw(&self) -> IoResult<String> {
        let cmdline = try!(self.read_proc("cmdline"));

        if cmdline == "" {
            return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "No cmdline present for process",
                detail: None
            });
        }

        return Ok(cmdline);
    }

    /// Return the arguments from `/proc/[pid]/cmdline` as a vector. If there
    /// are no arguments present in the file, it will return an Err instead of
    /// an empty vector.
    pub fn cmdline(&self) -> IoResult<Vec<String>> {
        let cmdline = try!(self.cmdline_raw());

        // Split terminator skips empty trailing substrings
        let split = cmdline.split_terminator(
            |&: c: char| c == '\0' || c == ' ');

        // `split` returns a vector of slices viewing `cmdline`, so they
        // get mapped to actuall strings before being returned as a vector.
        return Ok(split.map(|x| x.to_string()).collect());
    }

    /// Return the arguments from `/proc/[pid]/cmdline` as a string.
    pub fn cmdline_str(&self) -> IoResult<String> {
        return Ok(try!(self.cmdline_raw()).replace("\0", " "));
    }

    /// Return a name for the process. If possible, it will use the command line
    /// arguments for the process, falling back to using the name from
    /// `/proc/[pid]/stat` if there are none. Like `ps`, names are surrounded by
    /// square brackets if no command line arguments are available.
    pub fn extended_name(&self) -> String {
        return match self.cmdline_str() {
            Ok(cmdline) => cmdline,
            Err(_) => format!("[{}]", self.name.to_string())
        }
    }
}

/// Return a vector of all processes in /proc
pub fn all() -> Vec<Process> {
    let mut processes = Vec::new();

    for path in fs::readdir(&Path::new("/proc")).unwrap().iter() {
        match FromStr::from_str(path.filename_str().unwrap()) {
            Some(pid) => { processes.push(Process::new(pid).unwrap()) },
            None      => ()
        }
    }

    return processes;
}
