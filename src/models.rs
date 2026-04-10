#[derive(Clone, Default, Debug)]
pub struct CpuStats {
    pub total_usage: f32,
    pub cores_usage: Vec<f32>,
    pub frequency: u64,
    pub load: LoadStats,
    pub temperature: f32,
}

#[derive(Clone, Default, Debug)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub shared: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
}

#[derive(Clone, Default, Debug)]
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_speed: u64,
    pub write_speed: u64,
}

#[derive(Clone, Default, Debug)]
pub struct NetworkStats {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub interfaces: Vec<InterfaceStats>,
}

#[derive(Clone, Default, Debug)]
pub struct InterfaceStats {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Clone, Default, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub command: String,
    pub threads: u32,
    pub nice: i32,
    pub priority: i32,
    pub state: char,
}

#[derive(Clone, Default, Debug)]
pub struct LoadStats {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

#[derive(Clone, Default, Debug)]
pub struct SystemStats {
    pub uptime: f64,
    pub running_services: u32,
}
