mod cpu_times;
pub use cpu_times::*;

use crate::cpu::{CpuFreq, CpuStats};
use std::io;

pub fn cpu_stats() -> io::Result<CpuStats> {
	todo!();
}

pub fn cpu_freq() -> io::Result<CpuFreq> {
	todo!()
}

pub fn cpu_freq_percpu() -> io::Result<Vec<CpuFreq>> {
	todo!()
}
