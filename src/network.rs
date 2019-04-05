//! Load network informations

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use utils::read_file;

/// Struct that contains information about networks
#[derive(Clone, Copy, Debug)]
pub struct NetIOCounters {
    /// Number of bytes sent
    pub bytes_send: u64,

    /// Number of bytes received
    pub bytes_recv: u64,

    /// Number of packets sent
    pub packets_send: u64,

    /// Number of packets received
    pub packets_recv: u64,

    /// Total number of errors while receiving
    pub errin: u64,

    /// Total number of errors while sending
    pub errout: u64,

    /// Total number of incoming packets which were dropped
    pub dropin: u64,

    /// Total number of outgoing packets which were dropped (always 0 on macOS and BSD)
    pub dropout: u64,
}

/// Net counter struct to use nowrap mode
#[derive(Debug, Default)]
pub struct NetIOCountersCollector {
    /// Save the total of counters
    total_net_io_counters: HashMap<String, NetIOCounters>,

    /// Save the values of the last call of net_io_counters
    net_io_counters_last_statement: HashMap<String, NetIOCounters>,
}

impl NetIOCountersCollector {
    /// Reset de cache for net_io_counters in nowrap mode
    pub fn cache_clear(&mut self) {
        self.total_net_io_counters = HashMap::new();
        self.net_io_counters_last_statement = HashMap::new();
    }

    /// Return system-wide network I/O statistics as a Result of NetIOCounters
    ///
    /// If nowrap is true psutil will detect and adjust those numbers
    /// across function calls and add “old value” to “new value” so
    /// that the returned numbers will always be increasing or remain
    /// the same, but never decrease. net_io_counters.cache_clear() can
    /// be used to invalidate the nowrap cache.
    pub fn net_io_counters(&mut self, nowrap: bool) -> Result<NetIOCounters> {
        let total_net_io_counters = self.net_io_counters_pernic(nowrap)?;
        let mut final_net_io_counters = NetIOCounters {
            bytes_send: 0,
            bytes_recv: 0,
            packets_send: 0,
            packets_recv: 0,
            errin: 0,
            errout: 0,
            dropin: 0,
            dropout: 0,
        };
        for (_name, disk_io_counters) in total_net_io_counters {
            final_net_io_counters.bytes_send += disk_io_counters.bytes_send;
            final_net_io_counters.bytes_recv += disk_io_counters.bytes_recv;
            final_net_io_counters.packets_send += disk_io_counters.packets_send;
            final_net_io_counters.packets_recv += disk_io_counters.packets_recv;
            final_net_io_counters.errin += disk_io_counters.errin;
            final_net_io_counters.errout += disk_io_counters.errout;
            final_net_io_counters.dropin += disk_io_counters.dropin;
            final_net_io_counters.dropout += disk_io_counters.dropout;
        }
        Ok(final_net_io_counters)
    }

    /// Return for every network interface of the system the I/O statistics
    /// as Result of vector of String and NetIOCounters tuple.
    ///
    /// If nowrap is true psutil will detect and adjust those numbers
    /// across function calls and add “old value” to “new value” so
    /// that the returned numbers will always be increasing or remain
    /// the same, but never decrease. net_io_counters.cache_clear() can
    /// be used to invalidate the nowrap cache.
    pub fn net_io_counters_pernic(
        &mut self,
        nowrap: bool,
    ) -> Result<HashMap<String, NetIOCounters>> {
        let net_dev = read_file(Path::new("/proc/net/dev"))?;
        let mut net_lines: Vec<&str> = net_dev.lines().collect();
        // The two first lines contains no usefull informations
        net_lines.remove(1);
        net_lines.remove(0);
        let mut net_io_counters_vector = HashMap::new();
        for line in net_lines {
            let mut net_infos: Vec<&str> = line.split_whitespace().collect();
            let mut net_infos_u64: Vec<u64> = Vec::new();
            let mut net_name = String::from(net_infos[0]);
            net_name.pop();
            net_infos.remove(0);
            for net_info in net_infos {
                net_infos_u64.push(match net_info.parse::<u64>() {
                    Ok(u64_value) => u64_value,
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Impossible to parse '{}' in u64, error : {}",
                                net_info, error
                            ),
                        ));
                    }
                });
            }
            net_io_counters_vector.insert(
                net_name,
                NetIOCounters {
                    bytes_send: net_infos_u64[8],
                    bytes_recv: net_infos_u64[0],
                    packets_send: net_infos_u64[9],
                    packets_recv: net_infos_u64[1],
                    errin: net_infos_u64[2],
                    errout: net_infos_u64[10],
                    dropin: net_infos_u64[3],
                    dropout: net_infos_u64[11],
                },
            );
        }
        if nowrap {
            if self.net_io_counters_last_statement.is_empty() {
                self.total_net_io_counters = net_io_counters_vector.clone();
                self.net_io_counters_last_statement = net_io_counters_vector;
                Ok(self.total_net_io_counters.clone())
            } else {
                self.total_net_io_counters = total_net_io_counters(
                    &self.net_io_counters_last_statement,
                    &net_io_counters_vector,
                    &self.total_net_io_counters,
                );
                self.net_io_counters_last_statement = net_io_counters_vector;
                Ok(self.total_net_io_counters.clone())
            }
        } else {
            Ok(net_io_counters_vector)
        }
    }
}

/// Calculate nowrap for NetIOCounters field
fn nowrap(past_value: u64, current_value: u64, total_value: u64) -> u64 {
    const MAX_VALUE: u64 = 4_294_967_296;
    if current_value >= past_value {
        total_value + current_value - past_value
    } else {
        total_value + current_value + MAX_VALUE - past_value
    }
}

/// Calculate per interface the new NetIOCounters after a call of net_io_counters_pernic
fn total_net_io_counters(
    past_net_io_counters: &HashMap<String, NetIOCounters>,
    current_net_io_counters: &HashMap<String, NetIOCounters>,
    total_net_io_counters: &HashMap<String, NetIOCounters>,
) -> HashMap<String, NetIOCounters> {
    let mut final_net_io_counters: HashMap<String, NetIOCounters> = HashMap::new();
    for (name, current_counters) in current_net_io_counters {
        if past_net_io_counters.contains_key(name) && total_net_io_counters.contains_key(name) {
            let past_counters = past_net_io_counters[name];
            let total_counters = total_net_io_counters[name];
            final_net_io_counters.insert(
                name.clone(),
                NetIOCounters {
                    bytes_send: nowrap(
                        past_counters.bytes_send,
                        current_counters.bytes_send,
                        total_counters.bytes_send,
                    ),
                    bytes_recv: nowrap(
                        past_counters.bytes_recv,
                        current_counters.bytes_recv,
                        total_counters.bytes_recv,
                    ),
                    packets_send: nowrap(
                        past_counters.packets_send,
                        current_counters.packets_send,
                        total_counters.packets_send,
                    ),
                    packets_recv: nowrap(
                        past_counters.packets_recv,
                        current_counters.packets_recv,
                        total_counters.packets_recv,
                    ),
                    errin: nowrap(
                        past_counters.errin,
                        current_counters.errin,
                        total_counters.errin,
                    ),
                    errout: nowrap(
                        past_counters.errout,
                        current_counters.errout,
                        total_counters.errout,
                    ),
                    dropin: nowrap(
                        past_counters.dropin,
                        current_counters.dropin,
                        total_counters.dropin,
                    ),
                    dropout: nowrap(
                        past_counters.dropout,
                        current_counters.dropout,
                        total_counters.dropout,
                    ),
                },
            );
        } else {
            final_net_io_counters.insert(name.clone(), *current_counters);
        }
    }
    final_net_io_counters
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn total_net_io_counters_test() {
        //
        let mut past_net_io_counters = HashMap::new();
        past_net_io_counters.insert(
            String::from("eth0"),
            NetIOCounters {
                bytes_send: 3_000,
                bytes_recv: 10_000_000_000,
                packets_send: 4_000_000_000,
                packets_recv: 3_000_000_000,
                errin: 0,
                errout: 0,
                dropin: 0,
                dropout: 0,
            },
        );

        let mut current_net_io_counters = HashMap::new();
        current_net_io_counters.insert(
            String::from("eth0"),
            NetIOCounters {
                bytes_send: 5_000,
                bytes_recv: 50_000_000_000,  // 64 bits counter never wrap
                packets_send: 1_000_000_000, // 32 bits counter will wrap
                packets_recv: 2_000_000_000, // 32 bits counter will wrap
                errin: 0,
                errout: 0,
                dropin: 0,
                dropout: 0,
            },
        );
        current_net_io_counters.insert(
            String::from("eth1"),
            NetIOCounters {
                bytes_send: 42,
                bytes_recv: 0,
                packets_send: 0,
                packets_recv: 0,
                errin: 0,
                errout: 0,
                dropin: 0,
                dropout: 0,
            },
        );

        let mut total_net_io_counter = HashMap::new();
        total_net_io_counter.insert(
            String::from("eth0"),
            NetIOCounters {
                bytes_send: 3_000,
                bytes_recv: 10_000_000_000,
                packets_send: 4_000_000_000,
                packets_recv: 8_000_000_000, // 32 bits counter already wrap in the past
                errin: 0,
                errout: 0,
                dropin: 0,
                dropout: 0,
            },
        );

        let final_net_io_counters = total_net_io_counters(
            &past_net_io_counters,
            &current_net_io_counters,
            &total_net_io_counter,
        );

        assert_eq!(
            final_net_io_counters[&String::from("eth0")].bytes_send,
            5_000,
        );
        assert_eq!(
            final_net_io_counters[&String::from("eth0")].bytes_recv,
            50_000_000_000,
        );
        assert_eq!(
            final_net_io_counters[&String::from("eth0")].packets_send,
            4_294_967_296 + 1_000_000_000
        );
        assert_eq!(
            final_net_io_counters[&String::from("eth0")].packets_recv,
            8_000_000_000 + 1_294_967_296 + 2_000_000_000,
        );

        assert_eq!(final_net_io_counters[&String::from("eth1")].bytes_send, 42);
    }
}
