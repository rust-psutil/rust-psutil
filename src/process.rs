//! Read process-specific information from `/proc`

use std::old_io::fs;
use std::old_io::File;
use std::old_io::IoError;
use std::old_io::IoErrorKind;
use std::old_io::IoResult;
use std::slice::SliceConcatExt;
use std::str::FromStr;
use std::str::StrExt;
use std::vec::Vec;

/// Read a process' file from procfs - `/proc/[pid]/[name]`
fn procfs(pid: super::PID, name: &str) -> IoResult<String> {
    let mut path = Path::new("/proc");
    path.push(pid.to_string());
    path.push(name);
    return File::open(&path).read_to_string();
}

/// Possible statuses for a process
#[derive(Copy,Debug)]
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
#[derive(Debug)]
pub struct Process {
    pub pid: super::PID,
    pub name: String,
    pub status: Status
}

impl Process {
    /// Attempts to read process information from `/proc/[pid]/stat`.
    pub fn new(pid: super::PID) -> IoResult<Process> {
        let contents = try!(procfs(pid, "stat"));
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
    pub fn alive(&self) -> bool {
        match self.status {
            Status::Zombie => false,
            _ => true
        }
    }

    /// Return the arguments from `/proc/[pid]/cmdline` as a vector.
    ///
    /// Returns `Err` if `/proc/[pid]/cmdline` is empty.
    pub fn cmdline_vec(&self) -> IoResult<Vec<String>> {
        let cmdline = try!(procfs(self.pid, "cmdline"));

        if cmdline == "" {
            return Err(IoError {
                kind: IoErrorKind::InvalidInput,
                desc: "No cmdline present for process",
                detail: None
            });
        }

        // Split terminator skips empty trailing substrings
        let split = cmdline.split_terminator(
            |&: c: char| c == '\0' || c == ' ');

        // `split` returns a vector of slices viewing `cmdline`, so they
        // get mapped to actuall strings before being returned as a vector.
        return Ok(split.map(|x| x.to_string()).collect());
    }

    /// Return the result of `cmdline_vec` as a String
    pub fn cmdline(&self) -> IoResult<String> {
        return Ok(try!(self.cmdline_vec()).connect(" "));
    }

    /// Return a name for the process, using the same formatting as `ps`.
    ///
    /// If availible, it will return the command line arguments for the process,
    /// otherwise it will use the name from `/proc/[pid]/stat` enclosed with
    /// square brackets.
    pub fn extended_name(&self) -> String {
        return match self.cmdline() {
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
            Ok(pid) => { processes.push(Process::new(pid).unwrap()) },
            Err(_)  => ()
        }
    }

    return processes;
}
