//! Read process-specific information from `/proc`

use std::env::page_size;
use std::fs::read_dir;
use std::io::{Error,ErrorKind,Result};
use std::path::{Path,PathBuf};
use std::slice::SliceConcatExt;
use std::str::FromStr;
use std::str::StrExt;
use std::vec::Vec;

use ::PID;
use ::pidfile::read_pidfile;
use ::utils::read_file;

/// Read a process' file from procfs - `/proc/[pid]/[name]`
fn procfs(pid: super::PID, name: &str) -> Result<String> {
    let mut path = PathBuf::new("/proc");
    path.push(&pid.to_string());
    path.push(&name);
    return read_file(&path);
}

/// Possible statuses for a process
#[derive(Clone,Copy,Debug)]
pub enum State {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Traced,
    Paging
}

impl State {
    /// Returns a State based on a status character from `/proc/[pid]/stat`
    ///
    /// > One character from the string "RSDZTW" where R is running, S is
    /// > sleeping in an interruptible wait, D is waiting in uninterruptible
    /// > disk sleep, Z is zombie, T is traced or stopped (on a signal), and W
    /// > is paging.
    fn from_char(state: char) -> Result<Self> {
        match state {
            'R' => Ok(State::Running),
            'S' => Ok(State::Sleeping),
            'D' => Ok(State::Waiting),
            'Z' => Ok(State::Zombie),
            'T' => Ok(State::Traced),
            'W' => Ok(State::Paging),
             s  => Err(Error::new(ErrorKind::Other, "Invalid state character",
                    Some(format!("{} is not a known state", s))))
        }
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if !s.len() == 1 {
            Err(Error::new(ErrorKind::Other,
                "State must be a single character", None))
        } else {
            Ok(try!(State::from_char(s.char_at(0))))
        }
    }
}

/// Memory usage of a process
///
/// Read from `/proc/[pid]/statm`
#[derive(Copy,Debug)]
pub struct Memory {
    pub size: u64,
    pub resident: u64,
    pub share: u64,
    pub text: u64,
    pub lib: u64,
    pub data: u64,
    pub dt: u64
}

impl Memory {
    fn new(pid: PID) -> Result<Memory> {
        let statm = try!(procfs(pid, "statm"));
        let bytes: Vec<u64> = statm
            .trim_right()
            .split(" ")
            .map(|n| n.parse().unwrap())
            .collect();

        let page_size = page_size() as u64;

        return Ok(Memory {
            size:       bytes[0] * page_size,
            resident:   bytes[1] * page_size,
            share:      bytes[2] * page_size,
            text:       bytes[3] * page_size,
            lib:        bytes[4] * page_size,
            data:       bytes[5] * page_size,
            dt:         bytes[6] * page_size
        });
    }
}

/// A process with a PID
///
/// The manual pages for `proc` define integer sizes using `scanf(3)` format
/// specifiers, which parse to implementation specific sizes. This is obviously
/// a terrible idea, and so this code makes some assumptions about the sizes of
/// those specifiers.
///
/// These assumptions are backed up by `libc::types::os::arch::posix88::pid_t`,
/// which declares PIDs to be signed 32 bit integers. `proc(5)` declares that
/// PIDs use the `%d` format specifier.
///
/// - `%d` / `%u` - 32 bit integers
/// - `%ld` / `%lu` - 64 bit integers
///
/// *WARNING*: Rust currently has no support for 128 bit integers[1], so `%llu`
/// (used by the `starttime` and `delayacct_blkio_ticks` fields) is is instead
/// represented by a 64 bit integer, with the hope that doesn't break.
#[derive(Debug)]
pub struct Process {
    pub pid: PID,
    pub comm: String,
    pub state: State,
    pub ppid: PID,
    pub pgrp: i32,
    pub session: i32,
    pub tty_nr: i32,
    pub tpgid: i32,
    pub flags: u32,
    pub minflt: u64,
    pub cminflt: u64,
    pub majflt: u64,
    pub cmajflt: u64,
    pub utime: u64,
    pub stime: u64,
    pub cutime: i64,
    pub cstime: i64,
    pub priority: i64,
    pub nice: i64,
    pub num_threads: i64,
    pub itrealvalue: i64,
    pub starttime: u64,
    pub vsize: u64,
    pub rss: i64,
    pub rsslim: u64,
    pub startcode: u64,
    pub endcode: u64,
    pub startstack: u64,
    pub kstkesp: u64,
    pub kstkeip: u64,
    pub signal: u64,
    pub blocked: u64,
    pub sigignore: u64,
    pub sigcatch: u64,
    pub wchan: u64,
    pub nswap: u64,
    pub cnswap: u64,
    pub exit_signal: i32,
    pub processor: i32,
    pub rt_priority: u32,
    pub policy: u32,
    pub delayacct_blkio_ticks: u64,
    pub guest_time: u64,
    pub cguest_time: i64,
    pub start_data: u64,
    pub end_data: u64,
    pub start_brk: u64,
    pub arg_start: u64,
    pub arg_end: u64,
    pub env_start: u64,
    pub env_end: u64,
    pub exit_code: i32
}

/// TODO: This should use `try!` instead of `unwrap()`
macro_rules! from_str { ($field:expr) => (FromStr::from_str($field).unwrap()) }

impl Process {
    /// Parses a process name
    ///
    /// Process names are surrounded by `()` characters, which are removed.
    fn parse_comm(s: &str) -> String {
        let comm = s.to_string();
        comm[1..comm.len()-1].to_string()
    }

    /// Attempts to read process information from `/proc/[pid]/stat`.
    ///
    /// `/stat` is seperated by spaces and contains a trailing newline.
    ///
    /// This should return a psutil/process specific error type, so that  errors
    /// can be raised by `FromStr` too
    pub fn new(pid: PID) -> Result<Process> {
        let stat = try!(procfs(pid, "stat"));
        let stat: Vec<&str> = stat[0..stat.len()-1].split(' ').collect();

        // This may only be the case for Linux, but this can be removed or
        // changed when/if support for other kernels is needed
        if !stat.len() == 52 {
            return Err(Error::new(ErrorKind::Other,
                "Unexpected number of fields from /proc/[pid]/stat", None));
        }

        // Read each field into an attribute for a new Process instance
        return Ok(Process {
            pid:                    from_str!(stat[00]),
            comm:         Process::parse_comm(stat[01]),
            state:                  from_str!(stat[02]),
            ppid:                   from_str!(stat[03]),
            pgrp:                   from_str!(stat[04]),
            session:                from_str!(stat[05]),
            tty_nr:                 from_str!(stat[06]),
            tpgid:                  from_str!(stat[07]),
            flags:                  from_str!(stat[08]),
            minflt:                 from_str!(stat[09]),
            cminflt:                from_str!(stat[10]),
            majflt:                 from_str!(stat[11]),
            cmajflt:                from_str!(stat[12]),
            utime:                  from_str!(stat[13]),
            stime:                  from_str!(stat[14]),
            cutime:                 from_str!(stat[15]),
            cstime:                 from_str!(stat[16]),
            priority:               from_str!(stat[17]),
            nice:                   from_str!(stat[18]),
            num_threads:            from_str!(stat[19]),
            itrealvalue:            from_str!(stat[20]),
            starttime:              from_str!(stat[21]),
            vsize:                  from_str!(stat[22]),
            rss:                    from_str!(stat[23]),
            rsslim:                 from_str!(stat[24]),
            startcode:              from_str!(stat[25]),
            endcode:                from_str!(stat[26]),
            startstack:             from_str!(stat[27]),
            kstkesp:                from_str!(stat[28]),
            kstkeip:                from_str!(stat[29]),
            signal:                 from_str!(stat[30]),
            blocked:                from_str!(stat[31]),
            sigignore:              from_str!(stat[32]),
            sigcatch:               from_str!(stat[33]),
            wchan:                  from_str!(stat[34]),
            nswap:                  from_str!(stat[35]),
            cnswap:                 from_str!(stat[36]),
            exit_signal:            from_str!(stat[37]),
            processor:              from_str!(stat[38]),
            rt_priority:            from_str!(stat[39]),
            policy:                 from_str!(stat[40]),
            delayacct_blkio_ticks:  from_str!(stat[41]),
            guest_time:             from_str!(stat[42]),
            cguest_time:            from_str!(stat[43]),
            start_data:             from_str!(stat[44]),
            end_data:               from_str!(stat[45]),
            start_brk:              from_str!(stat[46]),
            arg_start:              from_str!(stat[47]),
            arg_end:                from_str!(stat[48]),
            env_start:              from_str!(stat[49]),
            env_end:                from_str!(stat[50]),
            exit_code:              from_str!(stat[51])
        });
    }

    /// Create a `Process` reading it's PID from a pidfile
    pub fn from_pidfile(path: &Path) -> Result<Process> {
        Process::new(try!(read_pidfile(&path)))
    }

    /// Return `true` if the process was alive at the time it was read
    pub fn is_alive(&self) -> bool {
        match self.state {
            State::Zombie => false,
            _ => true
        }
    }

    /// Read `/proc/[pid]/cmdline` as a vector.
    ///
    /// Returns `Err` if `/proc/[pid]/cmdline` is empty.
    pub fn cmdline_vec(&self) -> Result<Vec<String>> {
        let cmdline = try!(procfs(self.pid, "cmdline"));

        if cmdline == "" {
            // TODO: This should not use `std::io::Error`
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

    /// Reads `/proc/[pid]/statm` into a struct
    pub fn memory(&self) -> Result<Memory> {
        Memory::new(self.pid)
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
