use crate::models::ProcessInfo;
use std::fs;
use std::collections::HashMap;

pub struct ProcessManager {
    last_cpu_usage: HashMap<u32, (u64, u64)>, // pid -> (process_time, system_uptime_ticks)
    ticks_per_sec: u64,
}

impl ProcessManager {
    pub fn new() -> Self {
        let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as u64;
        Self {
            last_cpu_usage: HashMap::new(),
            ticks_per_sec: if ticks_per_sec > 0 { ticks_per_sec } else { 100 },
        }
    }

    pub fn get_process_list(&mut self) -> anyhow::Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();
        let uptime = crate::collectors::system::get_system_uptime();
        let uptime_ticks = (uptime * self.ticks_per_sec as f64) as u64;

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        if let Some(info) = self.parse_process(pid, uptime_ticks) {
                            processes.push(info);
                        }
                    }
                }
            }
        }
        
        let mut current_pids = HashMap::new();
        for p in &processes {
            current_pids.insert(p.pid, true);
        }
        self.last_cpu_usage.retain(|k, _| current_pids.contains_key(k));

        Ok(processes)
    }

    fn parse_process(&mut self, pid: u32, uptime_ticks: u64) -> Option<ProcessInfo> {
        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = fs::read_to_string(&stat_path).ok()?;
        
        let start_paren = stat_content.find('(')?;
        let end_paren = stat_content.rfind(')')?;
        
        if end_paren <= start_paren {
            return None;
        }

        let command = stat_content[start_paren + 1..end_paren].to_string();
        let after_paren = &stat_content[end_paren + 2..];
        let parts: Vec<&str> = after_paren.split_whitespace().collect();
        
        if parts.len() < 39 {
            return None;
        }

        let state = parts[0].chars().next().unwrap_or('?');
        let ppid: u32 = parts[1].parse().unwrap_or(0);
        let utime: u64 = parts[11].parse().unwrap_or(0);
        let stime: u64 = parts[12].parse().unwrap_or(0);
        let priority: i32 = parts[15].parse().unwrap_or(0);
        let nice: i32 = parts[16].parse().unwrap_or(0);
        let threads: u32 = parts[17].parse().unwrap_or(0);
        let rss: u64 = parts[21].parse().unwrap_or(0); // in pages

        let page_size = 4096;
        let memory_usage = rss * page_size;

        let process_time = utime + stime;
        
        let mut cpu_usage = 0.0;
        if let Some((last_time, last_uptime)) = self.last_cpu_usage.get(&pid) {
            let uptime_diff = uptime_ticks.saturating_sub(*last_uptime) as f32;
            let time_diff = process_time.saturating_sub(*last_time) as f32;
            if uptime_diff > 0.0 {
                cpu_usage = 100.0 * (time_diff / uptime_diff);
            }
        }
        
        self.last_cpu_usage.insert(pid, (process_time, uptime_ticks));

        let status_path = format!("/proc/{}/status", pid);
        let mut user = String::from("unknown");
        if let Ok(status) = fs::read_to_string(status_path) {
            for line in status.lines() {
                if line.starts_with("Uid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(uid) = parts[1].parse::<u32>() {
                            user = get_username(uid);
                        }
                    }
                    break;
                }
            }
        }

        Some(ProcessInfo {
            pid,
            ppid,
            user,
            cpu_usage,
            memory_usage,
            command,
            threads,
            nice,
            priority,
            state,
        })
    }

    pub fn kill_process(pid: u32) -> anyhow::Result<()> {
        unsafe {
            if libc::kill(pid as libc::pid_t, libc::SIGTERM) != 0 {
                return Err(anyhow::anyhow!("Failed to kill process"));
            }
        }
        Ok(())
    }

    pub fn renice_process(pid: u32, nice: i32) -> anyhow::Result<()> {
        unsafe {
            if libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, nice) != 0 {
                return Err(anyhow::anyhow!("Failed to renice process"));
            }
        }
        Ok(())
    }
}

fn get_username(uid: u32) -> String {
    let passwd = fs::read_to_string("/etc/passwd").unwrap_or_default();
    for line in passwd.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            if let Ok(id) = parts[2].parse::<u32>() {
                if id == uid {
                    return parts[0].to_string();
                }
            }
        }
    }
    uid.to_string()
}
