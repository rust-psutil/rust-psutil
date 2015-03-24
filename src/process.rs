//! Read process-specific information from `/proc`
//!
//! More information about specific fields can be found in
//! [proc(5)](http://man7.org/linux/man-pages/man5/proc.5.html).
//!
//! ### Field sizes
//!
//! The manual pages for `proc` define integer sizes using `scanf(3)` format
//! specifiers, which parse to implementation specific sizes. This is obviously
//! a terrible idea, and so this code makes some assumptions about the sizes of
//! those specifiers.
//!
//! These assumptions are backed up by `libc::types::os::arch::posix88::pid_t`,
//! which declares PIDs to be signed 32 bit integers. `proc(5)` declares that
//! PIDs use the `%d` format specifier.
//!
//! - `%d` / `%u` - 32 bit signed and unsigned integers
//! - `%ld` / `%lu` - 64 bit signed and unsigned integers
//!
//! **WARNING**: Rust currently has no support for 128 bit integers[2], so
//! `%llu` (used by the `starttime` and `delayacct_blkio_ticks` fields) is is
//! instead represented by a 64 bit integer, with the hope that doesn't break.
//!
//! ### CPU time fields and clock ticks
//!
//! The CPU time fields are very strange. Inside the Linux kernel they all use
//! the same type[1:L361], but when printed use different types[1:L456,L489] -
//! `utime`, `stime` and `gtime` are unsigned, whereas `cutime`, `cstime` and
//! `cgtime` are signed.
//!
//! These values are all returned as a number of clock ticks, which can be
//! divided by `sysconf(_SC_CLK_TCK)` to get a value in seconds. The `Process`
//! struct does this conversion automatically, and all CPU time fields use the
//! `f64` type.
//!
//! [1]: https://github.com/torvalds/linux/blob/4f671fe2f9523a1ea206f63fe60a7c7b3a56d5c7/fs/proc/array.c
//! [2]: https://github.com/rust-lang/rfcs/issues/521
//!

use std::env::page_size;
use std::fs::read_dir;
use std::io::{Error,ErrorKind,Result};
use std::path::{Path,PathBuf};
use std::slice::SliceConcatExt;
use std::str::FromStr;
use std::string::ToString;
use std::vec::Vec;

use libc::consts::os::sysconf::_SC_CLK_TCK;
use libc::funcs::posix88::unistd::sysconf;

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
                "State must be a single character",
                Some(format!("State string was: {}", s))))
        } else {
            State::from_char(s.char_at(0))
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            &State::Running  => "R".to_string(),
            &State::Sleeping => "S".to_string(),
            &State::Waiting  => "D".to_string(),
            &State::Zombie   => "Z".to_string(),
            &State::Traced   => "T".to_string(),
            &State::Paging   => "W".to_string()
        }
    }
}

/// Memory usage of a process
///
/// Read from `/proc/[pid]/statm`
#[derive(Copy,Debug)]
pub struct Memory {
    /// Total program size (bytes)
    pub size: u64,

    /// Resident Set Size (bytes)
    pub resident: u64,

    /// Shared pages (bytes)
    pub share: u64,

    /// Text
    pub text: u64,

    // /// Library (unused)
    // pub lib: u64,

    /// Data + stack
    pub data: u64,

    // /// Dirty pages (unused)
    // pub dt: u65
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
            // lib:     bytes[4] * page_size,
            data:       bytes[5] * page_size,
            // dt:      bytes[6] * page_size
        });
    }
}

/// Information about a process gathered from `/proc/[pid]/stat`.
///
/// **IMPORTANT**: See the module level notes for information on the types used
/// by this struct, as some do not match those used by `/proc/[pid]/stat`.
#[derive(Debug)]
pub struct Process {
    /// PID of the process
    pub pid: PID,

    /// Filename of the executable
    pub comm: String,

    /// State of the process as an enum
    pub state: State,

    /// PID of the parent process
    pub ppid: PID,

    /// Process group ID
    pub pgrp: i32,

    /// Session ID
    pub session: i32,

    /// Controlling terminal of the process [TODO: Actually two numbers]
    pub tty_nr: i32,

    /// ID of the foreground group of the controlling terminal
    pub tpgid: i32,

    /// Kernel flags for the process
    pub flags: u32,

    /// Minor faults
    pub minflt: u64,

    /// Minor faults by child processes
    pub cminflt: u64,

    /// Major faults
    pub majflt: u64,

    /// Major faults by child processes
    pub cmajflt: u64,

    /// Time scheduled in user mode (seconds)
    pub utime: f64,

    /// Time scheduled in kernel mode (seconds)
    pub stime: f64,

    /// Time waited-for child processes were scheduled in user mode (seconds)
    pub cutime: f64,

    /// Time waited-for child processes were scheduled in kernel mode (seconds)
    pub cstime: f64,

    /// Priority value (-100..-2 | 0..39)
    pub priority: i64,

    /// Nice value (-20..19)
    pub nice: i64,

    /// Number of threads in the process
    pub num_threads: i64,

    // /// Unmaintained field since linux 2.6.17, always 0
    // itrealvalue: i64,

    /// Time the process was started after system boot (clock ticks)
    pub starttime: u64,

    /// Virtual memory size in bytes
    pub vsize: u64,

    /// Resident Set Size (pages) [TODO: Calculate size in bytes]
    pub rss: i64,

    /// Current soft limit on process RSS (bytes)
    pub rsslim: u64,

    // These values are memory addresses
    startcode: u64,
    endcode: u64,
    startstack: u64,
    kstkesp: u64,
    kstkeip: u64,

    // /// Signal bitmaps
    // /// These are obselete, use `/proc/[pid]/status` instead
    // signal: u64,
    // blocked: u64,
    // sigignore: u64,
    // sigcatch: u64,

    /// Channel the process is waiting on (address of a system call)
    pub wchan: u64,

    // /// Number of pages swapped (not maintained)
    // pub nswap: u64,

    // /// Number of pages swapped for child processes (not maintained)
    // pub cnswap: u64,

    /// Signal sent to parent when process dies
    pub exit_signal: i32,

    /// Number of the CPU the process was last executed on
    pub processor: i32,

    /// Real-time scheduling priority (0 | 1..99)
    pub rt_priority: u32,

    /// Scheduling policy
    pub policy: u32,

    /// Aggregated block I/O delays (clock ticks)
    pub delayacct_blkio_ticks: u64,

    /// Guest time of the process (seconds)
    pub guest_time: f64,

    /// Guest time of the process's children (seconds)
    pub cguest_time: f64,

    // More memory addresses
    start_data: u64,
    end_data: u64,
    start_brk: u64,
    arg_start: u64,
    arg_end: u64,
    env_start: u64,
    env_end: u64,

    /// The thread's exit status
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

        // This is 'safe' to call as sysconf should only return an error for
        // invalid inputs, or options and limits (which _SC_CLK_TCK is not).
        let ticks_per_second: f64 = unsafe { sysconf(_SC_CLK_TCK) } as f64;

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
            utime:                  u64::from_str(stat[13]).unwrap() as f64 / ticks_per_second,
            stime:                  u64::from_str(stat[14]).unwrap() as f64 / ticks_per_second,
            cutime:                 i64::from_str(stat[15]).unwrap() as f64 / ticks_per_second,
            cstime:                 i64::from_str(stat[16]).unwrap() as f64 / ticks_per_second,
            priority:               from_str!(stat[17]),
            nice:                   from_str!(stat[18]),
            num_threads:            from_str!(stat[19]),
            // itrealvalue:         from_str!(stat[20]),
            starttime:              from_str!(stat[21]),
            vsize:                  from_str!(stat[22]),
            rss:                    from_str!(stat[23]),
            rsslim:                 from_str!(stat[24]),
            startcode:              from_str!(stat[25]),
            endcode:                from_str!(stat[26]),
            startstack:             from_str!(stat[27]),
            kstkesp:                from_str!(stat[28]),
            kstkeip:                from_str!(stat[29]),
            // signal:              from_str!(stat[30]),
            // blocked:             from_str!(stat[31]),
            // sigignore:           from_str!(stat[32]),
            // sigcatch:            from_str!(stat[33]),
            wchan:                  from_str!(stat[34]),
            // nswap:               from_str!(stat[35]),
            // cnswap:              from_str!(stat[36]),
            exit_signal:            from_str!(stat[37]),
            processor:              from_str!(stat[38]),
            rt_priority:            from_str!(stat[39]),
            policy:                 from_str!(stat[40]),
            delayacct_blkio_ticks:  from_str!(stat[41]),
            guest_time:             u64::from_str(stat[42]).unwrap() as f64 / ticks_per_second,
            cguest_time:            i64::from_str(stat[43]).unwrap() as f64 / ticks_per_second,
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

    /// Create a Process by reading it's PID from a pidfile.
    pub fn from_pidfile(path: &Path) -> Result<Process> {
        Process::new(try!(read_pidfile(&path)))
    }

    /// Return `true` if the process was alive at the time it was read.
    pub fn is_alive(&self) -> bool {
        match self.state {
            State::Zombie => false,
            _ => true
        }
    }

    /// Read `/proc/[pid]/cmdline` as a vector.
    ///
    /// Returns `Err` if `/proc/[pid]/cmdline` is empty.
    pub fn cmdline_vec(&self) -> Result<Option<Vec<String>>> {
        let cmdline = try!(procfs(self.pid, "cmdline"));

        if cmdline == "" {
            return Ok(None);
        } else {
            // Split terminator skips empty trailing substrings
            let split = cmdline.split_terminator(
                |c: char| c == '\0' || c == ' ');

            // `split` returns a vector of slices viewing `cmdline`, so they
            // get mapped to actuall strings before being returned as a vector.
            return Ok(Some(split.map(|x| x.to_string()).collect()));
        }
    }

    /// Return the result of `cmdline_vec` as a String.
    pub fn cmdline(&self) -> Result<Option<String>> {
        Ok(try!(self.cmdline_vec()).and_then(|c| Some(c.connect(" "))))
    }

    /// Reads `/proc/[pid]/statm` into a struct.
    pub fn memory(&self) -> Result<Memory> {
        Memory::new(self.pid)
    }

    /// Send SIGKILL to the process.
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
