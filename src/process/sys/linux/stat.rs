use std::convert::TryFrom;
use std::fs;
use std::io;
use std::str::FromStr;
use std::time::Duration;

use crate::process::{io_error_to_process_error, procfs_path, ProcessResult, Status};
use crate::utils::invalid_data;
use crate::{Pid, PAGE_SIZE, TICKS_PER_SECOND};

/// Returns a Status based on a status character from `/proc/[pid]/stat`.
///
/// See [array.c:115] and [proc(5)].
///
/// [array.c:115]: https://github.com/torvalds/linux/blob/master/fs/proc/array.c#L115
/// [proc(5)]: http://man7.org/linux/man-pages/man5/proc.5.html
impl TryFrom<char> for Status {
    type Error = std::io::Error;

    fn try_from(value: char) -> io::Result<Status> {
        match value {
            'R' => Ok(Status::Running),
            'S' => Ok(Status::Sleeping),
            'D' => Ok(Status::Waiting),
            'Z' => Ok(Status::Zombie),
            'T' => Ok(Status::Stopped),
            't' => Ok(Status::TracingStop),
            'X' | 'x' => Ok(Status::Dead),
            'K' => Ok(Status::WakeKill),
            'W' => Ok(Status::Waking),
            'P' => Ok(Status::Parked),
            'I' => Ok(Status::Idle),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid status character: {:?}", value),
            )),
        }
    }
}

impl FromStr for Status {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        if !s.len() == 1 {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Status is not a single character: {:?}", s),
            ))
        } else {
            Status::try_from(s.chars().nth(0).unwrap())
        }
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match *self {
            Status::Running => "R",
            Status::Sleeping => "S",
            Status::DiskSleep => "D",
            Status::Zombie => "Z",
            Status::Stopped => "T",
            Status::TracingStop => "t",
            Status::Dead => "X",
            Status::WakeKill => "K",
            Status::Waking => "W",
            Status::Parked => "P",
            Status::Idle => "I",
            _ => "",
        }
        .to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Stat {
    /// PID of the process.
    pub pid: Pid,

    /// Filename of the executable.
    pub comm: String,

    /// State of the process as an enum.
    pub state: Status,

    /// PID of the parent process.
    pub ppid: Option<Pid>,

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
    pub utime: Duration,

    /// Time scheduled in user mode (ticks).
    pub utime_ticks: u64,

    /// Time scheduled in kernel mode (seconds).
    pub stime: Duration,

    /// Time scheduled in kernel mode (ticks).
    pub stime_ticks: u64,

    /// Time waited-for child processes were scheduled in user mode (seconds).
    pub cutime: Duration,

    /// Time waited-for child processes were scheduled in user mode (ticks).
    pub cutime_ticks: i64,

    /// Time waited-for child processes were scheduled in kernel mode (seconds).
    pub cstime: Duration,

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
    pub starttime: Duration,

    /// Time the process was started after system boot (ticks).
    pub starttime_ticks: u128,

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
    pub delayacct_blkio: Duration,

    /// Aggregated block I/O delays (ticks).
    pub delayacct_blkio_ticks: u128,

    /// Guest time of the process (seconds).
    pub guest_time: Duration,

    /// Guest time of the process (ticks).
    pub guest_time_ticks: u64,

    /// Guest time of the process's children (seconds).
    pub cguest_time: Duration,

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

impl FromStr for Stat {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We parse the comm field and everything before it seperately since
        // the comm field is delimited by brackets and can contain spaces
        let (pid_field, leftover) = match s.find('(') {
            Some(i) => s.split_at(i - 1),
            None => return Err(invalid_data("Could not parse comm field")),
        };
        let (comm_field, leftover) = match leftover.rfind(')') {
            Some(i) => leftover.split_at(i + 2),
            None => return Err(invalid_data("Could not parse comm field")),
        };

        let mut fields: Vec<&str> = Vec::new();
        fields.push(pid_field);
        fields.push(&comm_field[2..comm_field.len() - 2]);
        fields.extend(leftover.trim_end().split_whitespace());

        if fields.len() != 52 {
            return Err(invalid_data(&format!(
                "Expected 52 fields, got {}",
                fields.len()
            )));
        }

        let pid = try_parse!(fields[0]);
        let comm = try_parse!(fields[1]);
        let state = try_parse!(fields[2]);

        let ppid = try_parse!(fields[3]);
        let ppid = if ppid == 0 { None } else { Some(ppid) };

        let pgrp = try_parse!(fields[4]);
        let session = try_parse!(fields[5]);
        let tty_nr = try_parse!(fields[6]);
        let tpgid = try_parse!(fields[7]);
        let flags = try_parse!(fields[8]);
        let minflt = try_parse!(fields[9]);
        let cminflt = try_parse!(fields[10]);
        let majflt = try_parse!(fields[11]);
        let cmajflt = try_parse!(fields[12]);

        let utime_ticks = try_parse!(fields[13]);
        let utime = Duration::from_secs_f64(utime_ticks as f64 / *TICKS_PER_SECOND);

        let stime_ticks = try_parse!(fields[14]);
        let stime = Duration::from_secs_f64(stime_ticks as f64 / *TICKS_PER_SECOND);

        let cutime_ticks = try_parse!(fields[15]);
        let cutime = Duration::from_secs_f64(cutime_ticks as f64 / *TICKS_PER_SECOND);

        let cstime_ticks = try_parse!(fields[16]);
        let cstime = Duration::from_secs_f64(cstime_ticks as f64 / *TICKS_PER_SECOND);

        let priority = try_parse!(fields[17]);
        let nice = try_parse!(fields[18]);
        let num_threads = try_parse!(fields[19]);
        // let itrealvalue = try_parse!(fields[20]);

        let starttime_ticks = try_parse!(fields[21]);
        let starttime = Duration::from_secs_f64(starttime_ticks as f64 / *TICKS_PER_SECOND);

        let vsize = try_parse!(fields[22]);
        let rss = try_parse!(fields[23], i64::from_str) * *PAGE_SIZE as i64;
        let rsslim = try_parse!(fields[24]);
        let startcode = try_parse!(fields[25]);
        let endcode = try_parse!(fields[26]);
        let startstack = try_parse!(fields[27]);
        let kstkesp = try_parse!(fields[28]);
        let kstkeip = try_parse!(fields[29]);
        let signal = try_parse!(fields[30]);
        let blocked = try_parse!(fields[31]);
        let sigignore = try_parse!(fields[32]);
        let sigcatch = try_parse!(fields[33]);
        let wchan = try_parse!(fields[34]);
        // let nswap = try_parse!(fields[35]);
        // let cnswap = try_parse!(fields[36]);
        let exit_signal = try_parse!(fields[37]);
        let processor = try_parse!(fields[38]);
        let rt_priority = try_parse!(fields[39]);
        let policy = try_parse!(fields[40]);

        let delayacct_blkio_ticks = try_parse!(fields[41]);
        let delayacct_blkio =
            Duration::from_secs_f64(delayacct_blkio_ticks as f64 / *TICKS_PER_SECOND);

        let guest_time_ticks = try_parse!(fields[42]);
        let guest_time = Duration::from_secs_f64(guest_time_ticks as f64 / *TICKS_PER_SECOND);

        let cguest_time_ticks = try_parse!(fields[43]);
        let cguest_time = Duration::from_secs_f64(cguest_time_ticks as f64 / *TICKS_PER_SECOND);

        let start_data = try_parse!(fields[44]);
        let end_data = try_parse!(fields[45]);
        let start_brk = try_parse!(fields[46]);
        let arg_start = try_parse!(fields[47]);
        let arg_end = try_parse!(fields[48]);
        let env_start = try_parse!(fields[49]);
        let env_end = try_parse!(fields[50]);
        let exit_code = try_parse!(fields[51]);

        Ok(Stat {
            pid,
            comm,
            state,
            ppid,
            pgrp,
            session,
            tty_nr,
            tpgid,
            flags,
            minflt,
            cminflt,
            majflt,
            cmajflt,
            utime,
            utime_ticks,
            stime,
            stime_ticks,
            cutime,
            cutime_ticks,
            cstime,
            cstime_ticks,
            priority,
            nice,
            num_threads,
            // itrealvalue,
            starttime,
            starttime_ticks,
            vsize,
            rss,
            rsslim,
            startcode,
            endcode,
            startstack,
            kstkesp,
            kstkeip,
            signal,
            blocked,
            sigignore,
            sigcatch,
            wchan,
            // nswap,
            // cnswap,
            exit_signal,
            processor,
            rt_priority,
            policy,
            delayacct_blkio,
            delayacct_blkio_ticks,
            guest_time,
            guest_time_ticks,
            cguest_time,
            cguest_time_ticks,
            start_data,
            end_data,
            start_brk,
            arg_start,
            arg_end,
            env_start,
            env_end,
            exit_code,
        })
    }
}

pub fn stat(pid: Pid) -> ProcessResult<Stat> {
    let data = fs::read_to_string(procfs_path(pid, "stat"))
        .map_err(|e| io_error_to_process_error(e, pid))?;

    Stat::from_str(&data).map_err(|e| io_error_to_process_error(e, pid))
}
