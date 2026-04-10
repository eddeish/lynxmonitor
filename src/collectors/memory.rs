use crate::models::MemoryStats;
use std::fs;

pub fn get_memory_usage() -> anyhow::Result<MemoryStats> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    let mut stats = MemoryStats::default();
    
    let mut mem_total = 0;
    let mut mem_free = 0;
    let mut buffers = 0;
    let mut cached = 0;
    let mut s_reclaimable = 0;
    let mut shmem = 0;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0];
        let val: u64 = parts[1].parse().unwrap_or(0) * 1024;

        match key {
            "MemTotal:" => mem_total = val,
            "MemFree:" => mem_free = val,
            "Buffers:" => buffers = val,
            "Cached:" => cached = val,
            "SReclaimable:" => s_reclaimable = val,
            "Shmem:" => shmem = val,
            "SwapTotal:" => stats.swap_total = val,
            "SwapFree:" => stats.swap_free = val,
            _ => {}
        }
    }

    stats.total = mem_total;
    stats.free = mem_free;
    stats.buffers = buffers;
    stats.cached = cached + s_reclaimable - shmem;
    stats.shared = shmem;
    stats.used = mem_total.saturating_sub(mem_free).saturating_sub(buffers).saturating_sub(stats.cached);
    stats.swap_used = stats.swap_total.saturating_sub(stats.swap_free);

    Ok(stats)
}
