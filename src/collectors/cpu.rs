use crate::models::{CpuStats, LoadStats};
use std::fs;

pub struct CpuCollector {
    last_total_user: u64,
    last_total_nice: u64,
    last_total_system: u64,
    last_total_idle: u64,
    last_total_iowait: u64,
    last_total_irq: u64,
    last_total_softirq: u64,
    last_total_steal: u64,
    last_cores: Vec<CoreData>,
}

#[derive(Default, Clone)]
struct CoreData {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            last_total_user: 0,
            last_total_nice: 0,
            last_total_system: 0,
            last_total_idle: 0,
            last_total_iowait: 0,
            last_total_irq: 0,
            last_total_softirq: 0,
            last_total_steal: 0,
            last_cores: Vec::new(),
        }
    }

    pub fn get_cpu_usage(&mut self) -> anyhow::Result<CpuStats> {
        let stat_content = fs::read_to_string("/proc/stat")?;
        let mut lines = stat_content.lines();
        
        let total_line = lines.next().unwrap_or_default();
        let total_parts: Vec<&str> = total_line.split_whitespace().collect();
        let mut total_usage = 0.0;
        
        if total_parts.len() > 8 && total_parts[0] == "cpu" {
            let user: u64 = total_parts[1].parse().unwrap_or(0);
            let nice: u64 = total_parts[2].parse().unwrap_or(0);
            let system: u64 = total_parts[3].parse().unwrap_or(0);
            let idle: u64 = total_parts[4].parse().unwrap_or(0);
            let iowait: u64 = total_parts[5].parse().unwrap_or(0);
            let irq: u64 = total_parts[6].parse().unwrap_or(0);
            let softirq: u64 = total_parts[7].parse().unwrap_or(0);
            let steal: u64 = total_parts[8].parse().unwrap_or(0);

            let past_idle = self.last_total_idle + self.last_total_iowait;
            let current_idle = idle + iowait;

            let past_non_idle = self.last_total_user + self.last_total_nice + self.last_total_system + self.last_total_irq + self.last_total_softirq + self.last_total_steal;
            let current_non_idle = user + nice + system + irq + softirq + steal;

            let past_total = past_idle + past_non_idle;
            let current_total = current_idle + current_non_idle;

            let total_diff = current_total.saturating_sub(past_total) as f32;
            let idle_diff = current_idle.saturating_sub(past_idle) as f32;

            if total_diff > 0.0 {
                total_usage = (total_diff - idle_diff) / total_diff * 100.0;
            }

            self.last_total_user = user;
            self.last_total_nice = nice;
            self.last_total_system = system;
            self.last_total_idle = idle;
            self.last_total_iowait = iowait;
            self.last_total_irq = irq;
            self.last_total_softirq = softirq;
            self.last_total_steal = steal;
        }

        let mut cores_usage = Vec::new();
        let mut current_cores = Vec::new();
        
        for line in lines {
            if line.starts_with("cpu") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 8 && parts[0] != "cpu" {
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let nice: u64 = parts[2].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle: u64 = parts[4].parse().unwrap_or(0);
                    let iowait: u64 = parts[5].parse().unwrap_or(0);
                    let irq: u64 = parts[6].parse().unwrap_or(0);
                    let softirq: u64 = parts[7].parse().unwrap_or(0);
                    let steal: u64 = parts[8].parse().unwrap_or(0);
                    
                    let core_data = CoreData {
                        user, nice, system, idle, iowait, irq, softirq, steal
                    };
                    current_cores.push(core_data.clone());
                }
            }
        }

        if self.last_cores.len() == current_cores.len() {
            for i in 0..current_cores.len() {
                let current = &current_cores[i];
                let past = &self.last_cores[i];

                let past_idle = past.idle + past.iowait;
                let current_idle = current.idle + current.iowait;

                let past_non_idle = past.user + past.nice + past.system + past.irq + past.softirq + past.steal;
                let current_non_idle = current.user + current.nice + current.system + current.irq + current.softirq + current.steal;

                let past_total = past_idle + past_non_idle;
                let current_total = current_idle + current_non_idle;

                let total_diff = current_total.saturating_sub(past_total) as f32;
                let idle_diff = current_idle.saturating_sub(past_idle) as f32;

                let db = if total_diff > 0.0 {
                    (total_diff - idle_diff) / total_diff * 100.0
                } else {
                    0.0
                };
                cores_usage.push(db);
            }
        }
        self.last_cores = current_cores;

        let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let mut frequency = 0;
        for line in cpuinfo.lines() {
            if line.starts_with("cpu MHz") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(freq) = val.trim().parse::<f64>() {
                        frequency = freq as u64;
                        break;
                    }
                }
            }
        }

        let load_content = fs::read_to_string("/proc/loadavg").unwrap_or_default();
        let load_parts: Vec<&str> = load_content.split_whitespace().collect();
        let mut load = LoadStats::default();
        if load_parts.len() >= 3 {
            load.one = load_parts[0].parse().unwrap_or(0.0);
            load.five = load_parts[1].parse().unwrap_or(0.0);
            load.fifteen = load_parts[2].parse().unwrap_or(0.0);
        }

        let temperature = get_cpu_temperature().unwrap_or(0.0);

        Ok(CpuStats {
            total_usage,
            cores_usage,
            frequency,
            load,
            temperature,
        })
    }
}

fn get_cpu_temperature() -> Option<f32> {
    if let Ok(entries) = fs::read_dir("/sys/class/thermal/") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.file_name().unwrap().to_string_lossy().starts_with("thermal_zone") {
                let temp_path = path.join("temp");
                if let Ok(val_str) = fs::read_to_string(temp_path) {
                    if let Ok(val) = val_str.trim().parse::<f32>() {
                        return Some(val / 1000.0);
                    }
                }
            }
        }
    }
    None
}
