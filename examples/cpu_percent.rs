use std::thread;
use std::time::Duration;

use psutil::cpu;

fn main() {
    let block_time = Duration::from_millis(1000);
    let mut cpu_percent_collector = cpu::CpuPercentCollector::new().unwrap();

    loop {
        thread::sleep(block_time);

        let cpu_percents = cpu_percent_collector.cpu_percent_percpu().unwrap();

        dbg!(cpu_percents);
    }
}
