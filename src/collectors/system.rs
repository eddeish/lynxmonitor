use std::fs;
use std::process::Command;

pub fn get_system_uptime() -> f64 {
    fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0)
}

pub fn get_running_services() -> u32 {
    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--state=running", "--no-pager", "--no-legend"])
        .output();
    
    match output {
        Ok(out) => {
            let output_str = String::from_utf8_lossy(&out.stdout);
            output_str.lines().count() as u32
        }
        Err(_) => 0,
    }
}
