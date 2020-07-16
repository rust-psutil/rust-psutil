use std::thread;
use std::time::Duration;

use psutil::process::processes;

// TODO: update to actually match the output of `ps aux`

fn main() {
	let processes = processes().unwrap();

	let block_time = Duration::from_millis(1000);
	thread::sleep(block_time);

	println!(
		"{:>6} {:>4} {:>4} {:.100}",
		"PID", "%CPU", "%MEM", "COMMAND"
	);
	for p in processes {
		match p {
			Ok(mut p) => {
				println!(
					"{:>6} {:>2.1} {:>2.1} {}",
					p.pid(),
					p.cpu_percent().unwrap_or(0.0f32),
					p.memory_percent().unwrap_or(0.0f32),
					p.cmdline()
						.unwrap_or(None)
						.unwrap_or_else(|| format!("[{}]", p.name().unwrap())),
				);
			}
			Err(_) => {}
		};
	}
}
