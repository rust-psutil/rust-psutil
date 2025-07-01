use std::io;
use std::path::PathBuf;

use glob::glob;

use crate::cpu::cpu_count;
use crate::cpu::CpuFreq;
use crate::types::FloatCount;
use crate::{read_file, Error, Result};

pub fn cpu_freq() -> io::Result<CpuFreq> {
	todo!()
}

fn cpu_get_cpuinfo_freq() -> Result<Vec<FloatCount>> {
	let cpuinfo_path = "/proc/cpuinfo";
	let contents = read_file(cpuinfo_path)?;
	contents
		.lines()
		.filter(|line| line.starts_with("cpu MHz"))
		.map(|line| {
			line.split(':')
				.next_back()
				.unwrap_or("0.0")
				.trim()
				.parse::<FloatCount>()
				.map_err(|err| Error::ParseFloat {
					path: cpuinfo_path.into(),
					contents: line.to_string(),
					source: err,
				})
		})
		.collect()
}

pub fn cpu_freq_percpu() -> io::Result<Vec<CpuFreq>> {
	let cpu_count = cpu_count() as usize;
	let cpuinfo_freqs = cpu_get_cpuinfo_freq().unwrap_or_default();
	let cpuinfo_freqs_valid = cpuinfo_freqs.len() == cpu_count;
	let mut paths: Vec<PathBuf> = glob("/sys/devices/system/cpu/cpufreq/policy[0-9]*")
		.expect("Failed to read constant glob pattern")
		.map(|path| path.expect("I'm not sure why this would ever fail"))
		.collect();

	if paths.is_empty() {
		paths = glob("/sys/devices/system/cpu/cpu[0-9]*/cpufreq")
			.expect("Failed to read glob pattern")
			.map(|path| path.expect("I'm not sure why this would ever fail"))
			.collect();
	}

	let mut sorted_paths: Vec<Option<PathBuf>> = vec![None; cpu_count];
	for path in paths {
		let path_string = path
			.clone()
			.into_os_string()
			.into_string()
			.expect("Path string was not valid utf-8");
		let cpu_num = path_string
			.chars()
			.filter(|c| c.is_ascii_digit())
			.collect::<String>()
			.parse::<usize>();
		if cpu_num.is_ok() {
			sorted_paths[cpu_num.unwrap()] = Some(path);
		}
	}

	let mut ret: Vec<CpuFreq> = vec![];
	for (i, path_entry) in sorted_paths.iter().enumerate() {
		match path_entry {
			Some(path) => {
				let curr = if cpuinfo_freqs_valid {
					cpuinfo_freqs[i] * 1000.0
				} else {
					read_file(path.join("scaling_cur_freq"))
						.unwrap_or_else(|_| {
							read_file(path.join("cpuinfo_cur_freq")).unwrap_or("0.0".to_string())
						})
						.trim()
						.parse::<f64>()
						.unwrap_or(0.0)
				};

				let curr = curr / 1000.0;
				let max = read_file(path.join("scaling_max_freq"))
					.unwrap()
					.trim()
					.parse::<f64>()
					.unwrap() / 1000.0;
				let min = read_file(path.join("scaling_min_freq"))
					.unwrap()
					.trim()
					.parse::<f64>()
					.unwrap() / 1000.0;
				ret.push(CpuFreq::new(curr, min, max));
			}
			None => {
				ret.push(CpuFreq::new(0.0, 0.0, 0.0));
			}
		}
	}

	Ok(ret)
}
