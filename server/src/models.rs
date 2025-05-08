use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PCInfo {
    pub pc_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_name: String,
    pub uptime: u64,
    pub process_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    pub interface_name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInfo {
    pub network_info: Vec<Network>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Disk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisksInfo {
    pub disksinfo: Vec<Disk>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OneCpuInfo {
    pub cpu_name: String,
    pub cpu_usage: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuInfo {
    pub cpu_info: Vec<OneCpuInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfoPayload {
    pub pc_info: PCInfo,
    pub system_info: SystemInfo,
    pub memory_info: MemoryInfo,
    pub network_info: NetworkInfo,
    pub disks_info: DisksInfo,
    pub cpu_info: CpuInfo,
}