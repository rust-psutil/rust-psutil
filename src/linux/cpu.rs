use std::collections::HashSet;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::{thread, time};

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
    let data = fs::read_to_string("/proc/stat")?;
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
    let data = fs::read_to_string("/proc/stat")?;
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
/// Use information contained in '/proc/stat'
pub fn cpu_percent(interval: f64) -> Result<f64> {
    Ok(cpu_times_percent(interval)?.busy_times())
}

/// Return the number of cores on the system (logical or physical, depending on the 'logical'
/// parameter).
///
/// Use information contained in '/proc/cpuinfo'
pub fn cpu_count(logical: bool) -> Result<u32> {
    let data = fs::read_to_string("/proc/cpuinfo")?;
    let (logical_cores, physical_cores) = cpu_count_inner(&data);
    if logical {
        Ok(logical_cores)
    } else {
        Ok(physical_cores)
    }
}

fn cpu_count_inner(data: &str) -> (u32, u32) {
    let mut logical_cores = 0;
    let mut physical_core_ids = HashSet::new();

    for line in data.lines() {
        if line.starts_with("processor") {
            logical_cores += 1;
        } else if line.starts_with("core id") {
            let fields: Vec<&str> = line.split_whitespace().collect();
            // Expect 4 tokens - 'core', 'id', ':', and the core ID itself
            if fields.len() == 4 {
                physical_core_ids.insert(fields[3]);
            }
        }
    }

    (logical_cores, physical_core_ids.len() as u32)
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

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn info_cpu_line_test() {
        let input = "cpu0 61286 322 19182 1708940 323 0 322 0 0 0 ";
        let result = match info_cpu_line(input) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };
        assert_eq!(
            result,
            vec![61286, 322, 19182, 1_708_940, 323, 0, 322, 0, 0, 0]
        );
    }

    #[test]
    fn cpu_line_to_cpu_times_test() {
        let input = vec![62972, 178, 18296, 349_198, 163, 0, 493, 0, 0, 0];
        let result = cpu_line_to_cpu_times(&input);
        assert_eq!(result.user, 62972);
        assert_eq!(result.nice, 178);
        assert_eq!(result.system, 18296);
        assert_eq!(result.idle, 349_198);
        assert_eq!(result.iowait, 163);
        assert_eq!(result.irq, 0);
        assert_eq!(result.softirq, 493);
        assert_eq!(result.steal, 0);
        assert_eq!(result.guest, 0);
        assert_eq!(result.guest_nice, 0);
    }

    #[test]
    fn cpu_time_percent_test() {
        let input1 = vec![62972, 178, 18296, 349_198, 163, 0, 493, 0, 0, 0];
        let input2 = vec![61286, 322, 19182, 1_708_940, 323, 0, 322, 0, 0, 0];
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

    #[test]
    fn cpu_count_test() {
        let data = "processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 58
model name	: Intel(R) Core(TM) i5-3320M CPU @ 2.60GHz
stepping	: 9
microcode	: 0x21
cpu MHz		: 3133.662
cache size	: 3072 KB
physical id	: 0
siblings	: 4
core id		: 0
cpu cores	: 2
apicid		: 0
initial apicid	: 0
fpu		: yes
fpu_exception	: yes
cpuid level	: 13
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx rdtscp lm constant_tsc arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx smx est tm2 ssse3 cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm cpuid_fault epb pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid fsgsbase smep erms xsaveopt dtherm ida arat pln pts md_clear flush_l1d
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds
bogomips	: 5187.82
clflush size	: 64
cache_alignment	: 64
address sizes	: 36 bits physical, 48 bits virtual
power management:

processor	: 1
vendor_id	: GenuineIntel
cpu family	: 6
model		: 58
model name	: Intel(R) Core(TM) i5-3320M CPU @ 2.60GHz
stepping	: 9
microcode	: 0x21
cpu MHz		: 3225.010
cache size	: 3072 KB
physical id	: 0
siblings	: 4
core id		: 0
cpu cores	: 2
apicid		: 1
initial apicid	: 1
fpu		: yes
fpu_exception	: yes
cpuid level	: 13
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx rdtscp lm constant_tsc arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx smx est tm2 ssse3 cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm cpuid_fault epb pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid fsgsbase smep erms xsaveopt dtherm ida arat pln pts md_clear flush_l1d
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds
bogomips	: 5187.82
clflush size	: 64
cache_alignment	: 64
address sizes	: 36 bits physical, 48 bits virtual
power management:
";
        let (logical, physical) = cpu_count_inner(data);
        assert_eq!(logical, 2);
        assert_eq!(physical, 1);
    }

    #[test]
    fn cpu_count_test_badlines() {
        let data = "processor|foobar
core id0
core id fj jfdsk fdslnfd dslkfj fjfdsj jfkfd
";
        // Just make sure it doesn't panic
        cpu_count_inner(data);
    }

}
