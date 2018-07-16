extern crate psutil;

use std::{thread, time};

fn main() {
    let mut disk_io_counters_collector = psutil::disk::DiskIOCountersCollector::default();

    loop {
        let past_disk_io_counters = match disk_io_counters_collector.disk_io_counters(true) {
            Ok(disk_io_counters) => disk_io_counters,
            Err(_) => {
                println!("Could not loading disk informations");
                continue;
            }
        };

        let block_time = time::Duration::from_millis(1000);
        thread::sleep(block_time);

        let current_disk_io_counters = match disk_io_counters_collector.disk_io_counters(true) {
            Ok(disk_io_counters) => disk_io_counters,
            Err(_) => {
                println!("Could not loading disk informations");
                continue;
            }
        };

        println!(
            "Disk general usage:
            read_bytes:         {} Bytes/s
            write_bytes:        {} Bytes/s
            ",
            (current_disk_io_counters.read_bytes - past_disk_io_counters.read_bytes),
            (current_disk_io_counters.write_bytes - past_disk_io_counters.write_bytes),
        );
    }
}
