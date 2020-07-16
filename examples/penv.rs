//! Print process environment

#[cfg(target_os = "linux")]
use psutil::process::os::linux::ProcessExt as _;
#[cfg(target_os = "windows")]
use psutil::process::os::windows::ProcessExt as _;
use psutil::process::Process;
use psutil::Pid;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	if args.len() != 2 {
		println!("PID not set");
		std::process::exit(1);
	}

	let pid = args[1].parse::<Pid>().expect("Invalid PID");
	match Process::new(pid) {
		Ok(process) => match process.environ() {
			Ok(env) => {
				for (k, v) in env.iter() {
					println!("{}={}", k, v);
				}
			}
			Err(e) => println!("{}", e),
		},
		Err(e) => println!("{}", e),
	}
}
