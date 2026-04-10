use crate::models::{NetworkStats, InterfaceStats};
use std::fs;

pub struct NetworkCollector {
    last_rx: u64,
    last_tx: u64,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            last_rx: 0,
            last_tx: 0,
        }
    }

    pub fn get_network_usage(&mut self) -> anyhow::Result<NetworkStats> {
        let mut stats = NetworkStats::default();
        let net_dev = fs::read_to_string("/proc/net/dev")?;
        
        let mut total_rx = 0;
        let mut total_tx = 0;

        for line in net_dev.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 17 {
                let name = parts[0].trim_end_matches(':').to_string();
                if name == "lo" {
                    continue;
                }
                
                let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                
                total_rx += rx_bytes;
                total_tx += tx_bytes;
                
                stats.interfaces.push(InterfaceStats {
                    name,
                    rx_bytes,
                    tx_bytes,
                });
            }
        }

        if self.last_rx > 0 {
            stats.download_speed = total_rx.saturating_sub(self.last_rx);
            stats.upload_speed = total_tx.saturating_sub(self.last_tx);
        }

        stats.download_bytes = total_rx;
        stats.upload_bytes = total_tx;

        self.last_rx = total_rx;
        self.last_tx = total_tx;

        Ok(stats)
    }
}
