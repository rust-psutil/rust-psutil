use std::thread;
use std::time::Duration;

use psutil::disk;

fn main() {
    let block_time = Duration::from_millis(1000);
    let mut disk_io_counters_collector = disk::DiskIoCountersCollector::default();
    let mut prev_disk_io_counters = disk_io_counters_collector.disk_io_counters().unwrap();

    loop {
        thread::sleep(block_time);

        let current_disk_io_counters = disk_io_counters_collector.disk_io_counters().unwrap();

        println!(
            "Disk general usage:
            read_bytes:         {} Bytes/s
            write_bytes:        {} Bytes/s
            ",
            (current_disk_io_counters.read_bytes() - prev_disk_io_counters.read_bytes()),
            (current_disk_io_counters.write_bytes() - prev_disk_io_counters.write_bytes()),
        );

        prev_disk_io_counters = current_disk_io_counters;
    }
}
