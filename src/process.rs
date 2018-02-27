//! Read process-specific information from `/proc`.

use std::fs::{self, read_dir, read_link};
use std::os::unix::fs::MetadataExt;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::vec::Vec;

use libc::{_SC_CLK_TCK, _SC_PAGESIZE, SIGKILL};
use libc::{kill, sysconf};

use {PID, UID, GID};
use pidfile::read_pidfile;
use utils::read_file;

lazy_static! {
    static ref TICKS_PER_SECOND: f64 = {
        unsafe { sysconf(_SC_CLK_TCK) as f64 }
    };
    static ref PAGE_SIZE: u64 = {
        unsafe { sysconf(_SC_PAGESIZE) as u64 }
    };
}

/// Return a path to a file in `/proc/[pid]/`.
fn procfs_path(pid: super::PID, name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("/proc");
    path.push(&pid.to_string());
    path.push(&name);
    path
}

/// Return an `io::Error` value and include the path in the message.
fn parse_error(message: &str, path: &PathBuf) -> Error {
    let path = path.to_str().unwrap_or("unknown path");
    Error::new(ErrorKind::InvalidInput,
               format!("{} (from {})", message, path))
}

/// Possible statuses for a process.
#[derive(Clone,Copy,Debug)]
pub enum State {
    Running,
    Sleeping,
    Waiting,
    Stopped,
    Traced,
    Paging,
    Dead,
    Zombie,
    Idle,
}

impl State {
    /// Returns a State based on a status character from `/proc/[pid]/stat`.
    ///
    /// See [array.c:115] and [proc(5)].
    ///
    /// [array.c:115]: https://github.com/torvalds/linux/blob/master/fs/proc/array.c#L115
    /// [proc(5)]: http://man7.org/linux/man-pages/man5/proc.5.html
    pub fn from_char(state: char) -> Result<Self> {
        match state {
            'R' => Ok(State::Running),
            'S' => Ok(State::Sleeping),
            'D' => Ok(State::Waiting),
            'T' => Ok(State::Stopped),
            't' => Ok(State::Traced),
            'W' => Ok(State::Paging),
            'Z' => Ok(State::Zombie),
            'X' => Ok(State::Dead),
            'I' => Ok(State::Idle),
            _ => Err(Error::new(ErrorKind::InvalidInput,
                                format!("Invalid state character: {:?}", state))),
        }
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if !s.len() == 1 {
            Err(Error::new(ErrorKind::InvalidInput,
                           format!("State is not a single character: {:?}", s)))
        } else {
            State::from_char(s.chars().nth(0).unwrap())
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match *self {
            State::Running => "R".to_string(),
            State::Sleeping => "S".to_string(),
            State::Waiting => "D".to_string(),
            State::Stopped => "T".to_string(),
            State::Traced => "t".to_string(),
            State::Paging => "W".to_string(),
            State::Zombie => "Z".to_string(),
            State::Dead => "X".to_string(),
            State::Idle => "I".to_string(),
        }
    }
}

/// Memory usage of a process read from `/proc/[pid]/statm`.
///
/// The `lib` [4, u64] and `dt` [6, u64] fields are ignored.
#[derive(Clone,Copy,Debug)]
pub struct Memory {
    /// Total program size (bytes).
    pub size: u64,

    /// Resident Set Size (bytes).
    pub resident: u64,

    /// Shared pages (bytes).
    pub share: u64,

    /// Text.
    pub text: u64,

    /// Data + stack.
    pub data: u64,
}

impl Memory {
    pub fn new(pid: PID) -> Result<Memory> {
        let path = procfs_path(pid, "statm");
        let statm = try!(read_file(&path));
        let fields: Vec<&str> = statm.trim_right().split(' ').collect();

        Ok(Memory {
            size: try!(Memory::parse_bytes(fields[0], &path)) * *PAGE_SIZE,
            resident: try!(Memory::parse_bytes(fields[1], &path)) * *PAGE_SIZE,
            share: try!(Memory::parse_bytes(fields[2], &path)) * *PAGE_SIZE,
            text: try!(Memory::parse_bytes(fields[3], &path)) * *PAGE_SIZE,
            data: try!(Memory::parse_bytes(fields[5], &path)) * *PAGE_SIZE,
        })
    }

    fn parse_bytes(field: &str, path: &PathBuf) -> Result<u64> {
        u64::from_str(field)
            .map_err(|e| parse_error(&format!("Could not parse memory: {}", e), path))
    }
}

pub struct Fd {
    /// Number of fd
    pub number: i32,

    /// Path to resolved fd
    pub path: PathBuf,

}


/// Information about a process gathered from `/proc/[pid]/stat`.
///
/// More information about specific fields can be found in [proc(5)].
///
/// # Field sizes
///
/// The manual pages for `proc` define integer sizes using `scanf(3)` format specifiers, which parse
/// to implementation specific sizes. This is obviously a terrible idea, and so this code makes some
/// assumptions about the sizes of those specifiers.
///
/// These assumptions are backed up by `libc::types::os::arch::posix88::pid_t`, which declares PIDs
/// to be signed 32 bit integers. `proc(5)` declares that PIDs use the `%d` format specifier.
///
/// - `%d` / `%u` - 32 bit signed and unsigned integers
/// - `%ld` / `%lu` - 64 bit signed and unsigned integers
///
/// Rust currently has no support for 128 bit integers ([rfc521]) so `%llu` (used by the `starttime`
/// and `delayacct_blkio_ticks` fields) is is instead represented by a 64 bit integer, with the hope
/// that doesn't break.
///
/// ### CPU time fields and clock ticks
///
/// The CPU time fields are very strange. Inside the Linux kernel they each use the same type
/// ([array.c:361]) but when printed use different types ([array.c:456]) - the fields `utime`,
/// `stime` and `gtime` are unsigned integers, whereas `cutime`, `cstime` and `cgtime` are signed
/// integers.
///
/// These values are all returned as a number of clock ticks, which can be divided by
/// `sysconf(_SC_CLK_TCK)` to get a value in seconds. The `Process` struct does this conversion
/// automatically, and all CPU time fields use the `f64` type. A corresponding `_ticks` field  (e.g.
/// `utime_ticks` gives the raw number of ticks as found in `/proc`.
///
/// # Unmaintained fields
///
/// The `itrealvalue` [20], `nswap` [35] and `cnswap` [36] fields are not maintained, and in some
/// cases are hardcoded to 0 in the kernel. The `signal` [30], `blocked` [31], `sigignore` [32] and
/// `sigcatch` [33] fields are included as private fields, but `proc(5)` recommends the use of
/// `/proc/[pid]/status` instead.
///
/// # Examples
///
/// ```
/// psutil::process::Process::new(psutil::getpid()).unwrap();
/// ```
///
/// [array.c:361]: https://github.com/torvalds/linux/blob/master/fs/proc/array.c#L361
/// [array.c:456]: https://github.com/torvalds/linux/blob/master/fs/proc/array.c#L456
/// [proc(5)]: http://man7.org/linux/man-pages/man5/proc.5.html
/// [rfc521]: https://github.com/rust-lang/rfcs/issues/521
#[derive(Clone,Debug)]
pub struct Process {
    /// PID of the process.
    pub pid: PID,

    /// UID of the process.
    pub uid: UID,

    /// UID of the process.
    pub gid: GID,

    /// Filename of the executable.
    pub comm: String,

    /// State of the process as an enum.
    pub state: State,

    /// PID of the parent process.
    pub ppid: PID,

    /// Process group ID.
    pub pgrp: i32,

    /// Session ID.
    pub session: i32,

    /// Controlling terminal of the process [TODO: Actually two numbers].
    pub tty_nr: i32,

    /// ID of the foreground group of the controlling terminal.
    pub tpgid: i32,

    /// Kernel flags for the process.
    pub flags: u32,

    /// Minor faults.
    pub minflt: u64,

    /// Minor faults by child processes.
    pub cminflt: u64,

    /// Major faults.
    pub majflt: u64,

    /// Major faults by child processes.
    pub cmajflt: u64,

    /// Time scheduled in user mode (seconds).
    pub utime: f64,

    /// Time scheduled in user mode (ticks).
    pub utime_ticks: u64,

    /// Time scheduled in kernel mode (seconds).
    pub stime: f64,

    /// Time scheduled in kernel mode (ticks).
    pub stime_ticks: u64,

    /// Time waited-for child processes were scheduled in user mode (seconds).
    pub cutime: f64,

    /// Time waited-for child processes were scheduled in user mode (ticks).
    pub cutime_ticks: i64,

    /// Time waited-for child processes were scheduled in kernel mode (seconds).
    pub cstime: f64,

    /// Time waited-for child processes were scheduled in kernel mode (ticks).
    pub cstime_ticks: i64,

    /// Priority value (-100..-2 | 0..39).
    pub priority: i64,

    /// Nice value (-20..19).
    pub nice: i64,

    /// Number of threads in the process.
    pub num_threads: i64,

    // Unmaintained field since linux 2.6.17, always 0.
    // itrealvalue: i64,
    /// Time the process was started after system boot (seconds).
    pub starttime: f64,

    /// Time the process was started after system boot (ticks).
    pub starttime_ticks: u64,

    /// Virtual memory size in bytes.
    pub vsize: u64,

    /// Resident Set Size (bytes).
    pub rss: i64,

    /// Current soft limit on process RSS (bytes).
    pub rsslim: u64,

    // These values are memory addresses
    startcode: u64,
    endcode: u64,
    startstack: u64,
    kstkesp: u64,
    kstkeip: u64,

    // Signal bitmaps.
    // These are obsolete, use `/proc/[pid]/status` instead.
    signal: u64,
    blocked: u64,
    sigignore: u64,
    sigcatch: u64,

    /// Channel the process is waiting on (address of a system call).
    pub wchan: u64,

    // Number of pages swapped (not maintained).
    // pub nswap: u64,
    //
    // Number of pages swapped for child processes (not maintained).
    // pub cnswap: u64,
    /// Signal sent to parent when process dies.
    pub exit_signal: i32,

    /// Number of the CPU the process was last executed on.
    pub processor: i32,

    /// Real-time scheduling priority (0 | 1..99).
    pub rt_priority: u32,

    /// Scheduling policy.
    pub policy: u32,

    /// Aggregated block I/O delays (seconds).
    pub delayacct_blkio: f64,

    /// Aggregated block I/O delays (ticks).
    pub delayacct_blkio_ticks: u64,

    /// Guest time of the process (seconds).
    pub guest_time: f64,

    /// Guest time of the process (ticks).
    pub guest_time_ticks: u64,

    /// Guest time of the process's children (seconds).
    pub cguest_time: f64,

    /// Guest time of the process's children (ticks).
    pub cguest_time_ticks: i64,

    // More memory addresses.
    start_data: u64,
    end_data: u64,
    start_brk: u64,
    arg_start: u64,
    arg_end: u64,
    env_start: u64,
    env_end: u64,

    /// The thread's exit status.
    pub exit_code: i32,
}

macro_rules! try_parse {
    ($field:expr) => {
        try_parse!($field, FromStr::from_str)
    };
    ($field:expr, $from_str:path) => {
        try!(match $from_str($field) {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new(ErrorKind::InvalidInput,
                format!("Could not parse {:?}", $field)))
        })
    };
}

impl Process {
    /// Attempts to read process information from `/proc/[pid]/stat`.
    ///
    /// Some additional metadata is read from the permissions on the `/proc/[pid]/`, which defines
    /// the process UID/GID. The format of `/proc/[pid]/stat` format is defined in proc(5).
    pub fn new(pid: PID) -> Result<Process> {
        let path = procfs_path(pid, "stat");
        let stat = try!(read_file(&path));
        let meta = try!(fs::metadata(procfs_path(pid, "")));
        Process::new_internal(&stat, meta.uid(), meta.gid(), &path)
    }

    fn new_internal(stat: &str, file_uid: UID, file_gid: GID, path: &PathBuf) -> Result<Process> {
        // Read the PID and comm fields separately, as the comm field is delimited by brackets and
        // could contain spaces.
        let (pid_, rest) = match stat.find('(') {
            Some(i) => stat.split_at(i - 1),
            None => return Err(parse_error("Could not parse comm", &path)),
        };
        let (comm, rest) = match rest.rfind(')') {
            Some(i) => rest.split_at(i + 2),
            None => return Err(parse_error("Could not parse comm", &path)),
        };

        // Split the rest of the fields on the space character and read them into a vector.
        let mut fields: Vec<&str> = Vec::new();
        fields.push(pid_);
        fields.push(&comm[2..comm.len() - 2]);
        fields.extend(rest.trim_right().split(' '));

        // Check we haven't read more or less fields than expected.
        if fields.len() != 52 {
            return Err(parse_error(&format!("Expected 52 fields, got {}", fields.len()), &path));
        }

        // Read each field into an attribute for a new Process instance
        Ok(Process {
            pid: try_parse!(fields[00]),
            uid: file_uid,
            gid: file_gid,
            comm: try_parse!(fields[1]),
            state: try_parse!(fields[2]),
            ppid: try_parse!(fields[3]),
            pgrp: try_parse!(fields[4]),
            session: try_parse!(fields[5]),
            tty_nr: try_parse!(fields[6]),
            tpgid: try_parse!(fields[7]),
            flags: try_parse!(fields[8]),
            minflt: try_parse!(fields[9]),
            cminflt: try_parse!(fields[10]),
            majflt: try_parse!(fields[11]),
            cmajflt: try_parse!(fields[12]),
            utime: try_parse!(fields[13], u64::from_str) as f64 / *TICKS_PER_SECOND,
            utime_ticks: try_parse!(fields[13], u64::from_str),
            stime: try_parse!(fields[14], u64::from_str) as f64 / *TICKS_PER_SECOND,
            stime_ticks: try_parse!(fields[14], u64::from_str),
            cutime: try_parse!(fields[15], i64::from_str) as f64 / *TICKS_PER_SECOND,
            cutime_ticks: try_parse!(fields[15], i64::from_str),
            cstime: try_parse!(fields[16], i64::from_str) as f64 / *TICKS_PER_SECOND,
            cstime_ticks: try_parse!(fields[16], i64::from_str),
            priority: try_parse!(fields[17]),
            nice: try_parse!(fields[18]),
            num_threads: try_parse!(fields[19]),
            // itrealvalue: try_parse!(fields[20]),
            starttime: try_parse!(fields[21], u64::from_str) as f64 / *TICKS_PER_SECOND,
            starttime_ticks: try_parse!(fields[21]),
            vsize: try_parse!(fields[22]),
            rss: try_parse!(fields[23], i64::from_str) * *PAGE_SIZE as i64,
            rsslim: try_parse!(fields[24]),
            startcode: try_parse!(fields[25]),
            endcode: try_parse!(fields[26]),
            startstack: try_parse!(fields[27]),
            kstkesp: try_parse!(fields[28]),
            kstkeip: try_parse!(fields[29]),
            signal: try_parse!(fields[30]),
            blocked: try_parse!(fields[31]),
            sigignore: try_parse!(fields[32]),
            sigcatch: try_parse!(fields[33]),
            wchan: try_parse!(fields[34]),
            // nswap: try_parse!(fields[35]),
            // cnswap: try_parse!(fields[36]),
            exit_signal: try_parse!(fields[37]),
            processor: try_parse!(fields[38]),
            rt_priority: try_parse!(fields[39]),
            policy: try_parse!(fields[40]),
            delayacct_blkio: try_parse!(fields[41], u64::from_str) as f64 / *TICKS_PER_SECOND,
            delayacct_blkio_ticks: try_parse!(fields[41]),
            guest_time: try_parse!(fields[42], u64::from_str) as f64 / *TICKS_PER_SECOND,
            guest_time_ticks: try_parse!(fields[42], u64::from_str),
            cguest_time: try_parse!(fields[43], i64::from_str) as f64 / *TICKS_PER_SECOND,
            cguest_time_ticks: try_parse!(fields[43], i64::from_str),
            start_data: try_parse!(fields[44]),
            end_data: try_parse!(fields[45]),
            start_brk: try_parse!(fields[46]),
            arg_start: try_parse!(fields[47]),
            arg_end: try_parse!(fields[48]),
            env_start: try_parse!(fields[49]),
            env_end: try_parse!(fields[50]),
            exit_code: try_parse!(fields[51]),
        })
    }

    /// Create a Process by reading its PID from a pidfile.
    pub fn from_pidfile(path: &Path) -> Result<Process> {
        Process::new(try!(read_pidfile(&path)))
    }

    /// Return `true` if the process was alive at the time it was read.
    pub fn is_alive(&self) -> bool {
        match self.state {
            State::Zombie => false,
            _ => true,
        }
    }

    /// Read `/proc/[pid]/cmdline` as a vector.
    ///
    /// Returns `Err` if `/proc/[pid]/cmdline` is empty.
    pub fn cmdline_vec(&self) -> Result<Option<Vec<String>>> {
        let cmdline = try!(read_file(&procfs_path(self.pid, "cmdline")));

        if cmdline.is_empty() {
            return Ok(None);
        }
        // Split terminator skips empty trailing substrings.
        let split = cmdline.split_terminator(|c: char| c == '\0' || c == ' ');

        // `split` returns a vector of slices viewing `cmdline`, so they
        // get mapped to actual strings before being returned as a vector.
        Ok(Some(split.map(|x| x.to_string()).collect()))
    }

    /// Return the result of `cmdline_vec` as a String.
    pub fn cmdline(&self) -> Result<Option<String>> {
        Ok(try!(self.cmdline_vec()).and_then(|c| Some(c.join(" "))))
    }

    /// Read the path of the process' current working directory.
    pub fn cwd(&self) -> Result<PathBuf> {
        read_link(procfs_path(self.pid, "cwd"))
    }

    /// Read the absolute path of the process
    pub fn exe(&self) -> Result<PathBuf> {
        read_link(procfs_path(self.pid, "exe"))
    }
    
    /// Reads `/proc/[pid]/statm` into a struct.
    pub fn memory(&self) -> Result<Memory> {
        Memory::new(self.pid)
    }

    /// Reads `/proc/[pid]/fd` directory
    pub fn open_fds(&self) -> Result<Vec<Fd>> {
        let mut fds = Vec::new();
        let entry_set = try!(read_dir(procfs_path(self.pid, "fd")));
        for entry in entry_set {
            if let Ok(entry) = entry {
                let path = entry.path();
                let fd_number =  try!(path.file_name().ok_or(parse_error("Could not read /proc entry", &path)));
                if let Ok(fd_path) = read_link(&path) {
                    fds.push(Fd{number: fd_number.to_string_lossy().parse::<i32>().unwrap(), path: fd_path})
                }
            }
        }

        Ok(fds)
    }

    /// Send SIGKILL to the process.
    pub fn kill(&self) -> Result<()> {
        match unsafe { kill(self.pid, SIGKILL) } {
            0 => Ok(()),
            -1 => Err(Error::last_os_error()),
            _ => unreachable!(),
        }
    }
}

impl PartialEq for Process {
    // Compares processes using their PID and `starttime` as an identity.
    fn eq(&self, other: &Process) -> bool {
        (self.pid == other.pid) && (self.starttime == other.starttime)
    }
}

/// Return a vector of all processes in `/proc`.
///
/// You may want to retry after a `std::io::ErrorKind::NotFound` error
/// due to a race condition where the contents of `/proc` are read,
/// and then a particular process dies before getting to read its info.
/// See example below.
///
/// ```ignore
/// loop {
///     match psutil::process::all() {
///         Ok(procs) => return Ok(procs),
///         Err(why) => {
///             if why.kind() != std::io::ErrorKind::NotFound {
///                 return Err(why);
///             }
///         }
///     }
/// }
/// ```
pub fn all() -> Result<Vec<Process>> {
    let mut processes = Vec::new();

    for entry in try!(read_dir(&Path::new("/proc"))) {
        let path = try!(entry).path();
        let name = try!(path.file_name().ok_or(parse_error("Could not read /proc entry", &path)));
        if let Ok(pid) = PID::from_str(&name.to_string_lossy()) {
            processes.push(try!(Process::new(pid)))
        }
    }

    Ok(processes)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn stat_52_fields() {
        let file_contents = "1 (init) S 0 1 1 0 -1 4219136 48162 38210015093 1033 16767427 1781 2205 119189638 18012864 20 0 1 0 9 34451456 504 18446744073709551615 1 1 0 0 0 0 0 4096 536962595 0 0 0 17 0 0 0 189 0 0 0 0 0 0 0 0 0 0\n";
        let p = Process::new_internal(&file_contents, 0, 0, &PathBuf::from("/proc/1/stat")).unwrap();
        assert_eq!(p.pid, 1);
        assert_eq!(p.comm, "init");
        assert_eq!(p.utime, 17.81);
    }

    #[test]
    fn starttime_in_seconds_and_ticks() {
        let file_contents = "1 (init) S 0 1 1 0 -1 4219136 48162 38210015093 1033 16767427 1781 2205 119189638 18012864 20 0 1 0 9 34451456 504 18446744073709551615 1 1 0 0 0 0 0 4096 536962595 0 0 0 17 0 0 0 189 0 0 0 0 0 0 0 0 0 0\n";
        let p = Process::new_internal(&file_contents, 0, 0, &PathBuf::from("/proc/1/stat")).unwrap();

        // This field should be in seconds
        if *TICKS_PER_SECOND == 100.0 {
            assert_eq!(p.starttime, 0.09);
        } else if *TICKS_PER_SECOND == 1000.0 {
            assert_eq!(p.starttime, 0.009);
        }
        assert_eq!((p.starttime * *TICKS_PER_SECOND) as u64, 9);


        // This field should be in clock ticks, i.e. the raw value from the file
        assert_eq!(p.starttime_ticks, 9);
    }
}
