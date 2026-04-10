use crate::models::DiskStats;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub struct DiskCollector {
    last_read_bytes: u64,
    last_write_bytes: u64,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            last_read_bytes: 0,
            last_write_bytes: 0,
        }
    }

    pub fn get_disk_usage(&mut self) -> anyhow::Result<DiskStats> {
        let mut stats = DiskStats::default();

        let statvfs = unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let mount_point = std::ffi::CString::new("/").unwrap();
            if libc::statvfs(mount_point.as_ptr(), &mut stat) == 0 {
                Some(stat)
            } else {
                None
            }
        };

        if let Some(s) = statvfs {
            stats.total = s.f_blocks as u64 * s.f_frsize as u64;
            stats.free = s.f_bavail as u64 * s.f_frsize as u64;
            stats.used = stats.total.saturating_sub(stats.free);
        }

        let diskstats = fs::read_to_string("/proc/diskstats").unwrap_or_default();
        let mut read_sectors = 0;
        let mut write_sectors = 0;

        for line in diskstats.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let dev_name = parts[2];
                if dev_name.starts_with("loop") || dev_name.starts_with("ram") {
                    continue;
                }
                let r_sect: u64 = parts[5].parse().unwrap_or(0);
                let w_sect: u64 = parts[9].parse().unwrap_or(0);
                read_sectors += r_sect;
                write_sectors += w_sect;
            }
        }

        let current_read_bytes = read_sectors * 512;
        let current_write_bytes = write_sectors * 512;

        if self.last_read_bytes > 0 {
            stats.read_speed = current_read_bytes.saturating_sub(self.last_read_bytes);
            stats.write_speed = current_write_bytes.saturating_sub(self.last_write_bytes);
        }

        stats.read_bytes = current_read_bytes;
        stats.write_bytes = current_write_bytes;

        self.last_read_bytes = current_read_bytes;
        self.last_write_bytes = current_write_bytes;

        Ok(stats)
    }
}
