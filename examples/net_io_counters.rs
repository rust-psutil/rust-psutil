extern crate psutil;

use std::{thread, time};

fn main() {
    let mut net_io_counters_collector = psutil::network::NetIOCountersCollector::default();

    loop {
        let past_net_io_counters = match net_io_counters_collector.net_io_counters(true) {
            Ok(net_io_counters) => net_io_counters,
            Err(error) => {
                println!("There is an error : {}", error.to_string());
                continue;
            }
        };

        let block_time = time::Duration::from_millis(1000);
        thread::sleep(block_time);

        let current_net_io_counters = match net_io_counters_collector.net_io_counters(true) {
            Ok(net_io_counters) => net_io_counters,
            Err(error) => {
                println!("{}", error.to_string());
                continue;
            }
        };

        println!(
            "Net general usage :
            bytes_send:     {} Bytes/s
            bytes_recv:     {} Bytes/s
            packets_send:   {} Packets/s
            packets_recv:   {} Packets/s
            ",
            (current_net_io_counters.bytes_send - past_net_io_counters.bytes_send),
            (current_net_io_counters.bytes_recv - past_net_io_counters.bytes_recv),
            (current_net_io_counters.packets_send - past_net_io_counters.packets_send),
            (current_net_io_counters.packets_recv - past_net_io_counters.packets_recv)
        )
    }
}
