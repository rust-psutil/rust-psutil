//! Read information about the operating system from `/proc`.

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str::FromStr;
use std::{thread, time};

use PID;

use utils::read_file;

#[derive(Debug)]
pub struct VirtualMemory {
    /// Amount of total memory
    pub total: u64,

    /// Amount of memory available for new processes
    pub available: u64,

    /// Percent of memory used
    pub percent: f32,

    /// Memory currently in use
    pub used: u64,

    /// Memory not being used
    pub free: u64,

    /// Memory currently in use
    pub active: u64,

    /// Memory that is not in use
    pub inactive: u64,

    /// Temporary storage for raw disk blocks
    pub buffers: u64,

    /// Memory used by the page cache
    pub cached: u64,

    /// Amount of memory consumed by tmpfs filesystems
    pub shared: u64,
}

impl VirtualMemory {
    pub fn new(
        total: u64,
        available: u64,
        shared: u64,
        free: u64,
        buffers: u64,
        cached: u64,
        active: u64,
        inactive: u64,
    ) -> VirtualMemory {
        let used = total - free - cached - buffers;

        VirtualMemory {
            total,
            available,
            shared,
            free,
            buffers,
            cached,
            active,
            inactive,
            used,
            percent: (total as f32 - available as f32) / total as f32 * 100.,
        }
    }
}

#[derive(Debug)]
pub struct SwapMemory {
    /// Amount of total swap memory
    pub total: u64,

    /// Amount of used swap memory
    pub used: u64,

    /// Amount of free swap memory
    pub free: u64,

    /// Percent of sway memory used
    pub percent: f32,

    /// Amount of memory swapped in from disk
    pub sin: u64,

    /// Amount of memory swapped to disk
    pub sout: u64,
}

impl SwapMemory {
    pub fn new(total: u64, free: u64, sin: u64, sout: u64) -> SwapMemory {
        let used = total - free;
        let percent = (used as f32 / total as f32) * 100.0;

        SwapMemory {
            total,
            used,
            free,
            percent,
            sin,
            sout,
        }
    }
}

#[derive(Debug)]
pub struct LoadAverage {
    /// number of jobs in the run queue averaged over 1 minute
    pub one: f32,

    /// number of jobs in the run queue averaged over 5 minute
    pub five: f32,

    /// number of jobs in the run queue averaged over 15 minute
    pub fifteen: f32,

    /// current number of runnable kernel entities
    pub runnable: i32,

    /// total number of runnable kernel entities
    pub total_runnable: i32,

    /// pid for the most recently created process
    pub last_pid: PID,
}

#[derive(Debug, Clone)]
pub struct CpuTimes {
    /// Time spent by normal processes executing in user mode;
    /// on Linux this also includes guest time
    pub user: u64,

    /// Time spent by niced (prioritized) processes executing in user mode;
    /// on Linux this also includes guest_nice time
    pub nice: u64,

    /// Time spent by processes executing in kernel mode
    pub system: u64,

    /// Time spent doing nothing
    pub idle: u64,

    /// Time spent waiting for I/O to complete
    pub iowait: u64,

    /// Time spent for servicing hardware interrupts
    pub irq: u64,

    /// Time spent for servicing software interrupts
    pub softirq: u64,

    /// Time spent by other operating systems running in a virtualized environment
    pub steal: u64,

    /// Time spent running a virtual CPU for guest operating systems
    /// under the control of the Linux kernel
    pub guest: u64,

    /// Time spent running a niced guest
    /// (virtual CPU for guest operating systems
    /// under the control of the Linux kernel)
    pub guest_nice: u64,
}

impl CpuTimes {
    /// Calculate the total time of CPU utilization.
    /// guest time and guest_nice time are respectively include
    /// on user and nice times
    fn total_time(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    /// Return the CpuTimesPercent object that contains the detailed
    /// CPU times percentage of CPU utilization between two instants.
    fn cpu_percent_since(&self, past_cpu_times: &CpuTimes) -> CpuTimesPercent {
        if self.total_time() > past_cpu_times.total_time() {
            let diff_total = (self.total_time() - past_cpu_times.total_time()) as f64;
            CpuTimesPercent {
                user: delta_percentage(self.user, past_cpu_times.user, diff_total),
                nice: delta_percentage(self.nice, past_cpu_times.nice, diff_total),
                system: delta_percentage(self.system, past_cpu_times.system, diff_total),
                idle: delta_percentage(self.idle, past_cpu_times.idle, diff_total),
                iowait: delta_percentage(self.iowait, past_cpu_times.iowait, diff_total),
                irq: delta_percentage(self.irq, past_cpu_times.irq, diff_total),
                softirq: delta_percentage(self.softirq, past_cpu_times.softirq, diff_total),
                steal: delta_percentage(self.steal, past_cpu_times.steal, diff_total),
                guest: delta_percentage(self.guest, past_cpu_times.guest, diff_total),
                guest_nice: delta_percentage(
                    self.guest_nice,
                    past_cpu_times.guest_nice,
                    diff_total,
                ),
            }
        } else {
            CpuTimesPercent {
                user: 0.,
                nice: 0.,
                system: 0.,
                idle: 0.,
                iowait: 0.,
                irq: 0.,
                softirq: 0.,
                steal: 0.,
                guest: 0.,
                guest_nice: 0.,
            }
        }
    }
}

#[derive(Debug)]
pub struct CpuTimesPercent {
    /// Percentage of time spent by normal processes executing in user mode
    /// between two instants;
    /// on Linux this also includes guest time
    pub user: f64,

    /// Percentage of time spent by niced (prioritized) processes
    /// executing in user mode between two instants;
    /// on Linux this also includes guest_nice time
    pub nice: f64,

    /// Percentage of time spent by processes executing in kernel
    /// mode between two instants
    pub system: f64,

    /// Percentage of time spent doing nothing between two instants
    pub idle: f64,

    /// Percentage of time spent waiting for I/O to complete between two instants
    pub iowait: f64,

    /// Percentage of time spent for servicing hardware interrupts
    /// between two instants
    pub irq: f64,

    /// Percentage of time spent for servicing software interrupts
    /// between two instants
    pub softirq: f64,

    /// Percentage of time spent by other operating systems running
    /// in a virtualized environment between two instants
    pub steal: f64,

    /// Percentage of time spent running a virtual CPU for guest operating systems
    /// under the control of the Linux kernel between two instants
    pub guest: f64,

    /// Percentage of time spent running a niced guest
    /// (virtual CPU for guest operating systems
    /// under the control of the Linux kernel) between two instants
    pub guest_nice: f64,
}

impl CpuTimesPercent {
    /// Caculculate the busy time in percent of a CPU
    /// Guest and guest_nice are count in user and nice.
    /// We ignore the CPU time in idle and iowait.
    fn busy_times(&self) -> f64 {
        self.user + self.nice + self.system + self.irq + self.softirq + self.steal
    }
}

/// To get a CpuPercent struct in non-blocking mode.
///
/// Example :
///
///     let mut cpu_percent_collector = match psutil::system::CpuPercentCollector::new() {
///         Ok(cpu_percent_collector) => cpu_percent_collector,
///         Err(_) => {
///             println!("Could not initialize cpu_percent_collector");
///             return;
///         },
///     };
///
///     // {... Your programme ...}
///
///     let cpu_times_percent = cpu_percent_collector.cpu_times_percent();
///     let cpu_times_percent_percpu = cpu_percent_collector.cpu_times_percent_percpu();
///
/// See an other example in examples/cpu_percent.
#[derive(Clone, Debug)]
pub struct CpuPercentCollector {
    /// Store the CPU times informations of the last call
    /// of class method cpu_times_percent or cpu_times_percent_percpu
    /// for the global CPU
    last_statement_cpu: CpuTimes,

    /// Store the CPU times informations of the last call
    /// of class method cpu_times_percent or cpu_times_percent_percpu per CPU.
    last_statement_percpu: Vec<CpuTimes>,
}

impl CpuPercentCollector {
    /// Initialize the CpuPercentCollector struct with the cpu_times informations.
    pub fn new() -> Result<CpuPercentCollector> {
        let last_statement_cpu = cpu_times()?;
        let last_statement_percpu = cpu_times_percpu()?;
        Ok(CpuPercentCollector {
            last_statement_cpu,
            last_statement_percpu,
        })
    }

    /// Returns a Result of CpuTimesPercent calculate
    /// since the last call of the method.
    /// For the first call it is since the object creation.
    ///
    /// The CpuTimesPercent object contains the detailed CPU utilization
    /// as percentage.
    pub fn cpu_times_percent(&mut self) -> Result<CpuTimesPercent> {
        let current_cpu_times = cpu_times()?;

        let cpu_percent_since = current_cpu_times.cpu_percent_since(&self.last_statement_cpu);

        self.last_statement_cpu = current_cpu_times;

        Ok(cpu_percent_since)
    }

    /// Returns a Result of vector of CpuTimesPercent
    /// calculate since the last call of the method.
    /// For the first call it is since the object creation.
    ///
    /// Each element of the vector reprensents the detailed
    /// CPU utilization as percentage per CPU.
    pub fn cpu_times_percent_percpu(&mut self) -> Result<Vec<CpuTimesPercent>> {
        let current_cpu_times_percpu = cpu_times_percpu()?;

        let mut cpu_times_percent_vector: Vec<CpuTimesPercent> = Vec::new();
        let current_cpu_times_percpu_copy = current_cpu_times_percpu.clone();
        for (iter, cpu_times) in current_cpu_times_percpu.iter().enumerate() {
            cpu_times_percent_vector
                .push(cpu_times.cpu_percent_since(&self.last_statement_percpu[iter]));
        }
        self.last_statement_percpu = current_cpu_times_percpu_copy;

        Ok(cpu_times_percent_vector)
    }
}

/// Returns the system uptime in seconds.
///
/// `/proc/uptime` contains the system uptime and idle time.
pub fn uptime() -> isize {
    let data = read_file(Path::new("/proc/uptime")).unwrap();
    uptime_internal(&data)
}

/// Returns the system uptime in seconds.
///
/// Input should be in the format '12489513.08 22906637.29\n'
fn uptime_internal(data: &str) -> isize {
    let numbers: Vec<&str> = data.split(' ').collect();
    let uptime: Vec<&str> = numbers[0].split('.').collect();
    FromStr::from_str(uptime[0]).unwrap()
}

/// Returns the system load average
///
/// `/proc/loadavg` contains the system load average
pub fn loadavg() -> Result<LoadAverage> {
    let data = read_file(Path::new("/proc/loadavg"))?;
    loadavg_internal(&data)
}

fn loadavg_internal(data: &str) -> Result<LoadAverage> {
    // strips off any trailing new line as well
    let fields: Vec<&str> = data.split_whitespace().collect();

    let one = try_parse!(fields[0]);
    let five = try_parse!(fields[1]);
    let fifteen = try_parse!(fields[2]);
    let last_pid = try_parse!(fields[4]);

    let entities = fields[3].split('/').collect::<Vec<&str>>();
    let runnable = try_parse!(entities[0]);
    let total_runnable = try_parse!(entities[1]);

    Ok(LoadAverage {
        one,
        five,
        fifteen,
        runnable,
        total_runnable,
        last_pid,
    })
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn uptime_parses() {
        assert_eq!(uptime_internal("12489513.08 22906637.29\n"), 12489513);
    }

    #[test]
    fn loadavg_parses() {
        let input = "0.49 0.70 0.84 2/519 1454\n";
        let out = loadavg_internal(input).unwrap();
        assert_eq!(out.one, 0.49);
        assert_eq!(out.five, 0.70);
        assert_eq!(out.fifteen, 0.84);
        assert_eq!(out.total_runnable, 519);
        assert_eq!(out.runnable, 2);
        assert_eq!(out.last_pid, 1454);
    }

    #[test]
    fn make_map_spaces() {
        let input = "field1: 23\nfield2: 45\nfield3: 100\n";
        let out = make_map(&input).unwrap();
        assert_eq!(out.get("field1:"), Some(&23));
        assert_eq!(out.get("field2:"), Some(&45));
    }

    #[test]
    fn make_map_tabs() {
        let input = "field1:\t\t\t45\nfield2:\t\t100\nfield4:\t\t\t\t4\n";
        let out = make_map(&input).unwrap();
        assert_eq!(out.get("field1:"), Some(&45));
        assert_eq!(out.get("field2:"), Some(&100));
    }

    #[test]
    fn make_map_with_ext() {
        let input = "field1: 100 kB\n field2: 200";
        let out = make_map(&input).unwrap();
        assert_eq!(out.get("field1:"), Some(&102400));
        assert_eq!(out.get("field2:"), Some(&200));
    }

    #[test]
    fn make_map_error() {
        let input = "field1: 2\nfield2: four";
        let out = make_map(&input);
        assert!(out.is_err())
    }

    #[test]
    fn multipler_kb() {
        assert_eq!(get_multiplier(&mut vec!["100", "kB"]), Some(1024));
    }

    #[test]
    fn multiplier_none() {
        assert_eq!(get_multiplier(&mut vec!["100", "200"]), None);
    }

    #[test]
    fn multiplier_last() {
        assert_eq!(
            get_multiplier(&mut vec!["100", "200", "400", "700", "kB"]),
            Some(1024)
        );
    }

    #[test]
    fn info_cpu_line_test() {
        let input = "cpu0 61286 322 19182 1708940 323 0 322 0 0 0 ";
        let result = match info_cpu_line(input) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(
            result,
            vec![61286, 322, 19182, 1708940, 323, 0, 322, 0, 0, 0]
        );
    }

    #[test]
    fn cpu_line_to_cpu_times_test() {
        let input = vec![62972, 178, 18296, 349198, 163, 0, 493, 0, 0, 0];
        let result = cpu_line_to_cpu_times(&input);
        assert_eq!(result.user, 62972);
        assert_eq!(result.nice, 178);
        assert_eq!(result.system, 18296);
        assert_eq!(result.idle, 349198);
        assert_eq!(result.iowait, 163);
        assert_eq!(result.irq, 0);
        assert_eq!(result.softirq, 493);
        assert_eq!(result.steal, 0);
        assert_eq!(result.guest, 0);
        assert_eq!(result.guest_nice, 0);
    }

    #[test]
    fn cpu_time_percent_test() {
        let input1 = vec![62972, 178, 18296, 349198, 163, 0, 493, 0, 0, 0];
        let input2 = vec![61286, 322, 19182, 1708940, 323, 0, 322, 0, 0, 0];
        let result1 = cpu_line_to_cpu_times(&input1);
        let result2 = cpu_line_to_cpu_times(&input2);
        let percent = result2.cpu_percent_since(&result1);
        assert!(percent.user >= 0.);
        assert!(percent.user <= 100.);
        assert!(percent.nice >= 0.);
        assert!(percent.nice <= 100.);
        assert!(percent.system >= 0.);
        assert!(percent.system <= 100.);
        assert!(percent.idle >= 0.);
        assert!(percent.idle <= 100.);
        assert!(percent.iowait >= 0.);
        assert!(percent.iowait <= 100.);
        assert!(percent.irq >= 0.);
        assert!(percent.irq <= 100.);
        assert!(percent.softirq >= 0.);
        assert!(percent.softirq <= 100.);
        assert!(percent.guest >= 0.);
        assert!(percent.guest <= 100.);
        assert!(percent.guest_nice >= 0.);
        assert!(percent.guest_nice <= 100.);
        assert!(percent.steal >= 0.);
        assert!(percent.steal <= 100.);
    }
}

fn not_found(key: &str) -> Error {
    Error::new(ErrorKind::NotFound, format!("{} not found", key))
}

/// Returns information about virtual memory usage
///
/// `/proc/meminfo` contains the virtual memory statistics
pub fn virtual_memory() -> Result<VirtualMemory> {
    let data = read_file(Path::new("/proc/meminfo"))?;
    let mem_info = make_map(&data)?;

    let total = *mem_info
        .get("MemTotal:")
        .ok_or_else(|| not_found("MemTotal"))?;
    let free = *mem_info
        .get("MemFree:")
        .ok_or_else(|| not_found("MemFree"))?;
    let buffers = *mem_info
        .get("Buffers:")
        .ok_or_else(|| not_found("Buffers"))?;
    let cached = *mem_info.get("Cached:").ok_or_else(|| not_found("Cached"))?
        // "free" cmdline utility sums reclaimable to cached.
        // Older versions of procps used to add slab memory instead.
        // This got changed in:
        //  https://gitlab.com/procps-ng/procps/commit/05d751c4f076a2f0118b914c5e51cfbb4762ad8e
        + *mem_info
            .get("SReclaimable:")
            .ok_or_else(|| not_found("SReclaimable"))?; // since kernel 2.6.19
    let active = *mem_info.get("Active:").ok_or_else(|| not_found("Active"))?;
    let inactive = *mem_info
        .get("Inactive:")
        .ok_or_else(|| not_found("Inactive"))?;

    // MemAvailable was introduced in kernel 3.14. The original psutil computes it if it's not
    // found, but since 3.14 has already reached EOL, let's assume that it's there.
    let available = *mem_info
        .get("MemAvailable:")
        .ok_or_else(|| not_found("MemAvailable"))?;

    // Shmem was introduced in 2.6.19
    let shared = *mem_info.get("Shmem:").ok_or_else(|| not_found("Shmem"))?;

    Ok(VirtualMemory::new(
        total, available, shared, free, buffers, cached, active, inactive,
    ))
}

/// Returns information about swap memory usage
///
/// `/proc/meminfo` and `/proc/vmstat` contains the information
pub fn swap_memory() -> Result<SwapMemory> {
    let data = read_file(Path::new("/proc/meminfo"))?;
    let swap_info = make_map(&data)?;

    let vmstat = read_file(Path::new("/proc/vmstat"))?;
    let vmstat_info = make_map(&vmstat)?;

    let total = *swap_info
        .get("SwapTotal:")
        .ok_or_else(|| not_found("SwapTotal"))?;
    let free = *swap_info
        .get("SwapFree:")
        .ok_or_else(|| not_found("SwapFree"))?;
    let sin = *vmstat_info
        .get("pswpin")
        .ok_or_else(|| not_found("pswpin"))?;
    let sout = *vmstat_info
        .get("pswpout")
        .ok_or_else(|| not_found("pswpout"))?;

    Ok(SwapMemory::new(total, free, sin, sout))
}

fn get_multiplier(fields: &mut Vec<&str>) -> Option<u64> {
    if let Some(ext) = fields.pop() {
        let multiplier = match ext {
            "kB" => Some(1024),
            _ => None,
        };
        fields.push(ext);

        multiplier
    } else {
        None
    }
}

fn make_map(data: &str) -> Result<HashMap<&str, u64>> {
    let lines: Vec<&str> = data.lines().collect();
    let mut map = HashMap::new();

    for line in lines {
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        let key = fields[0];
        let mut value = match fields[1].parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to parse {}", key),
                ));
            }
        };

        if let Some(multiplier) = get_multiplier(&mut fields) {
            value *= multiplier;
        }

        map.insert(key, value);
    }

    Ok(map)
}

/// Calculate a percentage from two values and a diff_total
///
/// Use in cpu_percent_since method
fn delta_percentage(current_value: u64, past_value: u64, total_diff: f64) -> f64 {
    if past_value <= current_value {
        let percentage = (100. * (current_value - past_value) as f64) / total_diff;
        if percentage > 100. {
            100.
        } else {
            percentage
        }
    } else {
        0.
    }
}

/// Test interval value validity
fn test_interval(interval: f64) -> Result<f64> {
    if interval <= 0. {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Interval must be greater than 0 : {}", interval),
        ))
    } else {
        Ok(interval)
    }
}

/// Convert a cpu line from /proc/stat into a Vec<u64>.
fn info_cpu_line(cpu_line: &str) -> Result<Vec<u64>> {
    let mut fields: Vec<&str> = cpu_line.split_whitespace().collect();
    if fields.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Wrong line use from /proc/stat : {}", cpu_line),
        ));
    }
    if fields.len() < 10 {
        return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Wrong format of /proc/stat line : {}, Maybe the kernel version is too old (Linux 2.6.33)", cpu_line),
        ));
    }
    // The first element of the line contains "cpux", we remove it.
    fields.remove(0);
    let mut values: Vec<u64> = Vec::new();
    for elt in fields {
        let value = match elt.parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to parse {}", elt),
                ));
            }
        };
        values.push(value);
    }
    Ok(values)
}

/// Return the CpuTimes object from the Vec<u64> obtain by info_cpu_line.
fn cpu_line_to_cpu_times(cpu_info: &[u64]) -> CpuTimes {
    let user = cpu_info[0];
    let nice = cpu_info[1];
    let system = cpu_info[2];
    let idle = cpu_info[3];
    let iowait = cpu_info[4];
    let irq = cpu_info[5];
    let softirq = cpu_info[6];
    let steal = cpu_info[7];
    let guest = cpu_info[8];
    let guest_nice = cpu_info[9];

    CpuTimes {
        user,
        nice,
        system,
        idle,
        iowait,
        irq,
        softirq,
        steal,
        guest,
        guest_nice,
    }
}

/// Returns information about cpu times usage.
///
/// `/proc/stat` contains the cpu times statistics
pub fn cpu_times() -> Result<CpuTimes> {
    let data = read_file(Path::new("/proc/stat"))?;
    let lines: Vec<&str> = data.lines().collect();
    let cpu_info = match info_cpu_line(lines[0]) {
        Ok(cpu_info) => cpu_info,
        Err(error) => return Err(error),
    };
    Ok(cpu_line_to_cpu_times(&cpu_info))
}

/// Returns information about cpu time usage on a Vec
/// that contains information per cpu
///
/// '/proc/stat' contains the cpu times statistics
pub fn cpu_times_percpu() -> Result<Vec<CpuTimes>> {
    let data = read_file(Path::new("/proc/stat"))?;
    let mut lines: Vec<&str> = data.lines().collect();
    let mut cpu_times_vector: Vec<CpuTimes> = Vec::new();
    // Remove the first line that contain the total cpu: "cpu"
    lines.remove(0);
    for line in lines {
        if line.starts_with("cpu") {
            let cpu_info = match info_cpu_line(line) {
                Ok(cpu_info) => cpu_info,
                Err(error) => return Err(error),
            };
            let cpu_time = cpu_line_to_cpu_times(&cpu_info);
            cpu_times_vector.push(cpu_time);
        }
    }
    Ok(cpu_times_vector)
}

/// Return a float representing the current system-wide
/// CPU utilization as percentage.
///
/// Interval must be > 0 seconds.
/// If interval < 0.1, the result of this function will be meaningless.
/// The function compares system CPU times elapsed before and after
/// the interval (blocking).
///
/// Use information contains in '/proc/stat'
pub fn cpu_percent(interval: f64) -> Result<f64> {
    Ok(cpu_times_percent(interval)?.busy_times())
}

/// Return a vector of floats representing the current system-wide
/// CPU utilization as percentage.
///
/// Interval must be > 0.0 seconds.
/// If interval < 0.1, the result of this function will be meaningless.
/// The function compares system CPU times per CPU elapsed before and after
/// the interval (blocking).
///
/// Use information contains in '/proc/stat'
pub fn cpu_percent_percpu(interval: f64) -> Result<Vec<f64>> {
    let cpu_percent_percpu = cpu_times_percent_percpu(interval)?;
    let mut cpu_percents: Vec<f64> = Vec::new();

    for cpu_percent in cpu_percent_percpu {
        cpu_percents.push(cpu_percent.busy_times());
    }
    Ok(cpu_percents)
}

/// Return a CpuTimesPercent representing the current detailed
/// CPU utilization as a percentage.
///
/// Interval must be > 0.0 seconds.
/// If interval is < 0.1, the result of this function will be meaningless.
/// The function compares all CPU times elapsed before and after
/// the interval (blocking).
///
/// Use informations contains in '/proc/stat'
pub fn cpu_times_percent(interval: f64) -> Result<CpuTimesPercent> {
    let mut cpu_percent_last_call = CpuPercentCollector::new()?;

    let interval = (test_interval(interval)? * 1000.) as u64;
    let block_time = time::Duration::from_millis(interval);
    thread::sleep(block_time);

    cpu_percent_last_call.cpu_times_percent()
}

/// Return a vector of CpuTimesPercent representing the current detailed
/// CPU utilization as percentage per cpu.
///
/// Interval must be > 0.0 seconds.
/// If interval is < 0.1, the result of this function will be meaningless.
/// The function compares all  CPU times per CPU elapsed before and after
/// the interval(blocking).
///
/// Use informations contains in '/proc/stat'
pub fn cpu_times_percent_percpu(interval: f64) -> Result<Vec<CpuTimesPercent>> {
    let mut cpu_percent_last_call = CpuPercentCollector::new()?;

    let interval = (test_interval(interval)? * 1000.) as u64;
    let block_time = time::Duration::from_millis(interval);
    thread::sleep(block_time);

    cpu_percent_last_call.cpu_times_percent_percpu()
}
