use std::{
	fs::{self, File},
	io::{self, BufRead, BufReader},
	num::ParseIntError,
	path::Path,
	str::FromStr,
};

use glob::glob;
use nix::NixPath;

use crate::{cpu::CpuFreq, Error, Result};

const PROC_CPUINFO: &str = "/proc/cpuinfo";

pub fn cpu_freq_percpu() -> Result<Vec<CpuFreq>> {
	if Path::new("/sys/devices/system/cpu/cpufreq/policy0").exists()
		|| Path::new("/sys/devices/system/cpu/cpu0/cpufreq").exists()
	{
		cpu_freq_percpu_sys_device()
	} else {
		cpu_freq_percpu_cpuinfo()
	}
}

fn cpu_freq_percpu_sys_device() -> Result<Vec<CpuFreq>> {
	let cpuinfo_freqs = cpu_freq_percpu_cpuinfo()?;

	let path = "/sys/devices/system/cpu/cpufreq/policy[0-9]*";
	let mut paths: Vec<_> = glob(path)
		// only errors on invalid pattern
		.unwrap()
		.collect::<std::result::Result<_, _>>()
		.map_err(|err| Error::ReadFile {
			path: path.into(),
			source: err.into_error(),
		})?;
	paths.sort_by_key(|path| {
		// this is scary, but it should be okay because we know the length of the path and that it's ascii
		std::str::from_utf8(&path.as_path().as_os_str().as_encoded_bytes()[38..])
			.unwrap()
			.parse::<usize>()
			.unwrap()
	});
	if paths.is_empty() {
		let path = "/sys/devices/system/cpu/cpu[0-9]*/cpufreq";
		paths = glob(path)
			// only errors on invalid pattern
			.unwrap()
			.collect::<std::result::Result<_, _>>()
			.map_err(|err| Error::ReadFile {
				path: path.into(),
				source: err.into_error(),
			})?;
		paths.sort_by_key(|path| {
			// this is scary, but it should be okay because we know the length of the path and that it's ascii
			std::str::from_utf8(&path.as_path().as_os_str().as_encoded_bytes()[27..path.len() - 8])
				.unwrap()
				.parse::<usize>()
				.unwrap()
		});
	}

	let mut result = Vec::new();
	let paths_len = paths.len();
	for (i, path) in paths.into_iter().enumerate() {
		let mut freq = if paths_len == cpuinfo_freqs.len() {
			// take cached value from cpuinfo if available, see:
			// https://github.com/giampaolo/psutil/issues/1851
			cpuinfo_freqs[i].clone()
		} else if let Ok(freq_ghz) = read_int::<u32, _>(path.join("scaling_cur_freq")) {
			CpuFreq {
				current: freq_ghz? as f64 / 1000.,
				min: None,
				max: None,
			}
		}
		// Likely an old RedHat, see:
		// https://github.com/giampaolo/psutil/issues/1071
		else if let Ok(freq_ghz) = read_int::<u32, _>(path.join("cpuinfo_cur_freq")) {
			CpuFreq {
				current: freq_ghz? as f64 / 1000.,
				min: None,
				max: None,
			}
		} else if matches!(
			fs::read_to_string(format!("/sys/devices/system/cpu/cpu{i}/online")).as_deref(),
			Ok("0\n")
		) {
			result.push(CpuFreq {
				current: 0.0,
				min: None,
				max: None,
			});
			continue;
		} else {
			// give up, we don't know how to get frequency
			todo!("can't find current frequency file")
		};

		freq.min = read_int(path.join("scaling_min_freq"))
			.ok()
			.transpose()?
			.map(|x: u32| x as f64 / 1000.);
		freq.max = read_int(path.join("scaling_max_freq"))
			.ok()
			.transpose()?
			.map(|x: u32| x as f64 / 1000.);
		result.push(freq);
	}

	Ok(result)
}

/// Return current CPU frequency from cpuinfo if available.
fn cpu_freq_percpu_cpuinfo() -> Result<Vec<CpuFreq>> {
	let mut result = Vec::new();
	for line in BufReader::new(File::open(PROC_CPUINFO)?).lines() {
		let line = line?;
		if line.to_lowercase().starts_with("cpu mhz") {
			let freq = line
				.split_once(':')
				.ok_or_else(|| Error::MissingData {
					path: PROC_CPUINFO.into(),
					contents: line.to_string(),
				})?
				.1
				.trim()
				.parse()
				.map_err(|err| Error::ParseFloat {
					path: PROC_CPUINFO.into(),
					contents: line.to_string(),
					source: err,
				})?;

			result.push(CpuFreq {
				current: freq,
				min: None,
				max: None,
			});
		}
	}

	Ok(result)
}

fn read_int<T, P>(path: P) -> io::Result<Result<T>>
where
	P: AsRef<Path>,
	T: FromStr<Err = ParseIntError>,
{
	let text = fs::read_to_string(path.as_ref())?;
	Ok(text.trim().parse().map_err(|err| Error::ParseInt {
		path: path.as_ref().to_owned(),
		contents: text,
		source: err,
	}))
}
