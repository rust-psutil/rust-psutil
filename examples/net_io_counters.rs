use std::thread;
use std::time::Duration;

use psutil::network;

fn main() {
	let block_time = Duration::from_millis(1000);
	let mut net_io_counters_collector = network::NetIoCountersCollector::default();
	let mut prev_net_io_counters = net_io_counters_collector.net_io_counters().unwrap();

	loop {
		thread::sleep(block_time);

		let current_net_io_counters = net_io_counters_collector.net_io_counters().unwrap();

		println!(
			"Net general usage :
            bytes_send:     {} Bytes/s
            bytes_recv:     {} Bytes/s
            packets_send:   {} Packets/s
            packets_recv:   {} Packets/s
            ",
			(current_net_io_counters.bytes_sent() - prev_net_io_counters.bytes_sent()),
			(current_net_io_counters.bytes_recv() - prev_net_io_counters.bytes_recv()),
			(current_net_io_counters.packets_sent() - prev_net_io_counters.packets_sent()),
			(current_net_io_counters.packets_recv() - prev_net_io_counters.packets_recv())
		);

		prev_net_io_counters = current_net_io_counters;
	}
}
