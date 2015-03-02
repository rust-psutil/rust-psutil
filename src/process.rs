//! Read process-specific information from `/proc`

use std::fs::read_dir;
use std::io::Write;
use std::io::{Error,ErrorKind,Result};
use std::path::{Path,PathBuf};
use std::slice::SliceConcatExt;
use std::str::FromStr;
use std::str::StrExt;
use std::vec::Vec;

use ::utils::read_file;

use super::pidfile::read_pidfile;

/// Read a process' file from procfs - `/proc/[pid]/[name]`
fn procfs(pid: super::PID, name: &str) -> Result<String> {
    let mut path = PathBuf::new("/proc");
    path.push(&pid.to_string());
    path.push(&name);
    return read_file(&path);
}

/// Memory usage of a process
///
/// Currently Linux specific.
#[cfg(target_os="linux")]
#[derive(Copy,Debug)]
pub struct Memory {
    pub rss: i32,
    pub vms: i32,
    pub shared: i32,
    pub text: i32,
    pub lib: i32,
    pub data: i32,
    pub dirty: i32
}

/// Possible statuses for a process
#[derive(Clone,Copy,Debug)]
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
#[derive(Clone,Debug)]
pub struct Process {
    pub pid: super::PID,
    pub name: String,
    pub status: Status
}

impl Process {
    /// Attempts to read process information from `/proc/[pid]/stat`.
    pub fn new(pid: super::PID) -> Result<Process> {
        let contents = try!(procfs(pid, "stat"));
        let stat: Vec<&str> = contents.split(' ').collect();

        // This may only be the case for Linux, but this can be removed or
        // changed when/if support for other kernels is needed
        assert!(stat.len() == 52, "Unknown stat fields");

        // Read each field into an attribute for a new Process instance
        return Ok(Process {
            pid: FromStr::from_str(stat[0]).unwrap(),
            name: {
                // The process name is surrounded by () becuase why not
                let name = stat[1].to_string();
                name[1..name.len()-1].to_string()
            },
            status: Status::from_char(stat[2].chars().next().unwrap())
        });
    }

    /// Call `Process::new()`, reading the PID from a pidfile
    pub fn new_from_pidfile(path: &Path) -> Result<Process> {
        Process::new(try!(read_pidfile(&path)))
    }

    /// Return `true` if the process is/was alive (at the time it was read).
    #[unstable]
    pub fn alive(&self) -> bool {
        match self.status {
            Status::Zombie => false,
            _ => true
        }
    }

    /// Return the arguments from `/proc/[pid]/cmdline` as a vector.
    ///
    /// Returns `Err` if `/proc/[pid]/cmdline` is empty.
    pub fn cmdline_vec(&self) -> Result<Vec<String>> {
        let cmdline = try!(procfs(self.pid, "cmdline"));

        if cmdline == "" {
            return Err(Error::new(
                ErrorKind::Other, "No cmdline present for process", None));
        }

        // Split terminator skips empty trailing substrings
        let split = cmdline.split_terminator(
            |c: char| c == '\0' || c == ' ');

        // `split` returns a vector of slices viewing `cmdline`, so they
        // get mapped to actuall strings before being returned as a vector.
        return Ok(split.map(|x| x.to_string()).collect());
    }

    /// Return the result of `cmdline_vec` as a String
    pub fn cmdline(&self) -> Result<String> {
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

    /// Reads `/proc/[pid]/statm` into a struct
    ///
    /// **TODO**: `i32` might not be big enough
    #[cfg(target_os="linux")]
    pub fn memory(&self) -> Result<Memory> {
        let statm = try!(procfs(self.pid, "statm"));
        let bytes: Vec<i32> = statm
            .trim_right()
            .split(" ")
            .map(|n| n.parse().unwrap())
            .collect();

        return Ok(Memory {
            rss:    bytes[0],
            vms:    bytes[1],
            shared: bytes[2],
            text:   bytes[3],
            lib:    bytes[4],
            data:   bytes[5],
            dirty:  bytes[6]
        });
    }

    /// Send SIGKILL to the process
    pub fn kill(&self) -> Result<()> {
        use libc::funcs::posix88::signal::kill;
        use libc::consts::os::posix88::SIGKILL;

        return match unsafe { kill(self.pid, SIGKILL) } {
            0  => Ok(()),
            -1 => Err(Error::last_os_error()),
            _  => unreachable!()
        };
    }
}

/// Return a vector of all processes in /proc
pub fn all() -> Vec<Process> {
    let mut processes = Vec::new();

    for entry in read_dir(&Path::new("/proc")).unwrap() {
        let path = entry.unwrap().path();
        let file_name = path.file_name().unwrap();
        match FromStr::from_str(&file_name.to_string_lossy()) {
            Ok(pid) => { processes.push(Process::new(pid).unwrap()) },
            Err(_)  => ()
        }
    }

    return processes;
}
