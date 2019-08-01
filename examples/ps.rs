use psutil::process::processes;

// TODO: update to actually match the output of `ps aux`

fn main() {
    println!(
        "{:>5} {:^5} {:>8} {:>8} {:.100}",
        "PID", "STATE", "UTIME", "STIME", "CMD"
    );

    for p in processes().unwrap() {
        let p = p.unwrap();
        let cpu_times = p.cpu_times().unwrap();

        println!(
            "{:>5} {:^5?} {:>8.2?} {:>8.2?} {:.100}",
            p.pid(),
            p.status().unwrap(),
            cpu_times.user(),
            cpu_times.system(),
            p.cmdline()
                .unwrap()
                .unwrap_or_else(|| format!("[{}]", p.name().unwrap())),
        );
    }
}
