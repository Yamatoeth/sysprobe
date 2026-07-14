#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: NetworkInfo,
    pub top_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuInfo {
    pub global_usage_percent: f32,
    pub per_core_usage_percent: Vec<f32>,
    pub load_avg_1: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f32,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub active_connections: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}
