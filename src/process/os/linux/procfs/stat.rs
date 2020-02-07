use std::fs;
use std::io;
use std::str::FromStr;
use std::time::Duration;

use crate::process::{io_error_to_process_error, procfs_path, ProcessResult, Status};
use crate::utils::invalid_data;
use crate::{Pid, PAGE_SIZE, TICKS_PER_SECOND};

/// New struct, not in Python psutil.
#[derive(Clone, Debug)]
pub struct ProcfsStat {
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

	/// Unmaintained field since linux 2.6.17, always 0.
	pub itrealvalue: i64,

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
	pub policy: u64,

	/// Aggregated block I/O delays (seconds).
	pub delayacct_blkio: Option<Duration>,

	/// Aggregated block I/O delays (ticks).
	pub delayacct_blkio_ticks: Option<u128>,

	/// Guest time of the process (seconds).
	pub guest_time: Option<Duration>,

	/// Guest time of the process (ticks).
	pub guest_time_ticks: Option<u64>,

	/// Guest time of the process's children (seconds).
	pub cguest_time: Option<Duration>,

	/// Guest time of the process's children (ticks).
	pub cguest_time_ticks: Option<i64>,

	// More memory addresses.
	start_data: Option<u64>,
	end_data: Option<u64>,
	start_brk: Option<u64>,
	arg_start: Option<u64>,
	arg_end: Option<u64>,
	env_start: Option<u64>,
	env_end: Option<u64>,

	/// The thread's exit status.
	pub exit_code: Option<i32>,
}

impl FromStr for ProcfsStat {
	type Err = io::Error;

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

		if fields.len() < 41 {
			return Err(invalid_data(&format!(
				"Expected at least 41 fields, got {}",
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
		let itrealvalue = try_parse!(fields[20]);

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

		// since kernel 2.6.18
		let delayacct_blkio_ticks = if fields.len() >= 42 {
			Some(try_parse!(fields[41]))
		} else {
			None
		};
		let delayacct_blkio = delayacct_blkio_ticks
			.map(|val| Duration::from_secs_f64(val as f64 / *TICKS_PER_SECOND));

		// since kernel 2.6.24
		let (guest_time_ticks, cguest_time_ticks) = if fields.len() >= 44 {
			(Some(try_parse!(fields[42])), Some(try_parse!(fields[43])))
		} else {
			(None, None)
		};
		let guest_time =
			guest_time_ticks.map(|val| Duration::from_secs_f64(val as f64 / *TICKS_PER_SECOND));
		let cguest_time =
			cguest_time_ticks.map(|val| Duration::from_secs_f64(val as f64 / *TICKS_PER_SECOND));

		// since kernel 3.3
		let (start_data, end_data, start_brk) = if fields.len() >= 47 {
			(
				Some(try_parse!(fields[44])),
				Some(try_parse!(fields[45])),
				Some(try_parse!(fields[46])),
			)
		} else {
			(None, None, None)
		};

		// since kernel 3.5
		let (arg_start, arg_end, env_start, env_end, exit_code) = if fields.len() >= 52 {
			(
				Some(try_parse!(fields[47])),
				Some(try_parse!(fields[48])),
				Some(try_parse!(fields[49])),
				Some(try_parse!(fields[50])),
				Some(try_parse!(fields[51])),
			)
		} else {
			(None, None, None, None, None)
		};

		Ok(ProcfsStat {
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
			itrealvalue,
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

/// New function, not in Python psutil.
pub fn procfs_stat(pid: Pid) -> ProcessResult<ProcfsStat> {
	let data = fs::read_to_string(procfs_path(pid, "stat"))
		.map_err(|e| io_error_to_process_error(e, pid))?;

	ProcfsStat::from_str(&data).map_err(|e| io_error_to_process_error(e, pid))
}
