use clap::Parser;
use std::time::{Duration, Instant};

mod models;
mod collectors;
mod process;
mod ui;
mod utils;

use collectors::{
    cpu::CpuCollector,
    memory::get_memory_usage,
    disk::DiskCollector,
    network::NetworkCollector,
    system::{get_system_uptime, get_running_services},
};
use models::{CpuStats, MemoryStats, DiskStats, NetworkStats};
use process::manager::ProcessManager;
use ui::engine::UiEngine;
use utils::logger::Logger;

#[derive(Parser, Debug)]
#[command(author, version, about = "LynxMonitor — Modern Linux System Monitor", long_about = None)]
struct Args {
    #[arg(long, default_value_t = 1000, help = "Refresh interval in milliseconds")]
    refresh: u64,

    #[arg(long, default_value_t = false, help = "Disable GPU panel")]
    no_gpu: bool,

    #[arg(long, default_value = "cpu", help = "Sort processes by: cpu, mem, pid")]
    sort: String,

    #[arg(long, default_value = "", help = "Filter processes by user or command")]
    filter: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut logger = Logger::new();
    logger.info("main", "Starting LynxMonitor");

    let mut cpu_collector = CpuCollector::new();
    let mut disk_collector = DiskCollector::new();
    let mut net_collector = NetworkCollector::new();
    let mut process_manager = ProcessManager::new();

    let mut ui = UiEngine::new(!args.no_gpu, args.sort.clone(), args.filter.clone())?;
    let refresh_duration = Duration::from_millis(args.refresh);
    let mut last_tick = Instant::now();

    let mut cached_cpu = CpuStats::default();
    let mut cached_mem = MemoryStats::default();
    let mut cached_disk = DiskStats::default();
    let mut cached_net = NetworkStats::default();
    let mut cached_procs = Vec::new();
    let mut cached_uptime = 0.0;
    let mut cached_services = 0;

    loop {
        if last_tick.elapsed() >= refresh_duration {
            cached_cpu = cpu_collector.get_cpu_usage().unwrap_or_default();
            cached_mem = get_memory_usage().unwrap_or_default();
            cached_disk = disk_collector.get_disk_usage().unwrap_or_default();
            cached_net = net_collector.get_network_usage().unwrap_or_default();
            cached_procs = process_manager.get_process_list().unwrap_or_default();
            cached_uptime = get_system_uptime();
            cached_services = get_running_services();
            last_tick = Instant::now();
        }

        ui.draw(
            &cached_cpu,
            &cached_mem,
            &cached_disk,
            &cached_net,
            &cached_procs,
            cached_uptime,
            cached_services,
        )?;

        match ui.handle_input(&cached_procs)? {
            ui::engine::InputResult::Quit => break,
            ui::engine::InputResult::Kill(pid) => {
                let _ = ProcessManager::kill_process(pid);
                logger.info("main", &format!("Sent SIGTERM to PID {}", pid));
            }
            ui::engine::InputResult::Continue => {}
        }
    }

    ui.cleanup()?;
    Ok(())
}
