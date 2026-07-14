#[cfg(target_os = "linux")]
use crate::collectors::network_proc::NetworkProcCollector;
use crate::collectors::Collector;
use crate::snapshot::NetworkInfo;

pub struct NetworkCollector;

impl Collector for NetworkCollector {
    type Output = NetworkInfo;

    fn collect(&mut self, _sys: &mut sysinfo::System) -> anyhow::Result<Self::Output> {
        let mut networks = sysinfo::Networks::new_with_refreshed_list();
        networks.refresh();

        let rx_bytes_per_sec = networks.values().map(sysinfo::NetworkData::received).sum();
        let tx_bytes_per_sec = networks
            .values()
            .map(sysinfo::NetworkData::transmitted)
            .sum();

        Ok(NetworkInfo {
            rx_bytes_per_sec,
            tx_bytes_per_sec,
            active_connections: active_connection_count(),
        })
    }
}

fn active_connection_count() -> usize {
    #[cfg(target_os = "linux")]
    {
        NetworkProcCollector::active_connection_count().unwrap_or(0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}
