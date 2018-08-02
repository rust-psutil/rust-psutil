extern crate psutil;

use std::{thread, time};

fn main() {
    let mut cpu_percent_collector = match psutil::system::CpuPercentCollector::new() {
        Ok(cpu_percent_collector) => cpu_percent_collector,
        Err(_) => {
            println!("Could not initialize the CpuPercentCollector object");
            return;
        }
    };
    loop {
        let block_time = time::Duration::from_millis(200);
        thread::sleep(block_time);

        let cpu_times_percent = match cpu_percent_collector.cpu_times_percent() {
            Ok(cpu_times_percent) => cpu_times_percent,
            Err(_) => {
                println!("Could not loading Cpu info");
                continue;
            }
        };

        println!(
            "%Cpu(s): {:.2} us, {:.2} sy, {:.2} ni, {:.2} id, {:.2} wa, {:.2} hi, {:.2} si, {:.2} st",
            cpu_times_percent.user,
            cpu_times_percent.system,
            cpu_times_percent.nice,
            cpu_times_percent.idle,
            cpu_times_percent.iowait,
            cpu_times_percent.irq,
            cpu_times_percent.softirq,
            cpu_times_percent.steal
        );
    }
}
