use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PCInfo {
    pub pc_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_name: String,
    pub uptime: u64,
    pub process_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Network {
    pub interface_name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInfo {
    pub network_info: Vec<Network>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Disk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisksInfo {
    pub disksinfo: Vec<Disk>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OneCpuInfo {
    pub cpu_name: String,
    pub cpu_usage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CpuInfo {
    pub cpu_info: Vec<OneCpuInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfoPayload {
    pub server_id: String,
    pub pc_info: PCInfo,
    pub system_info: SystemInfo,
    pub memory_info: MemoryInfo,
    pub network_info: NetworkInfo,
    pub disks_info: DisksInfo,
    pub cpu_info: CpuInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerData {
    pub id: String,
    pub pc_name: String,
    pub ip_address: Option<String>,
    pub last_seen: String,
    pub online: bool,
    pub system_info: Option<SystemInfo>,
    pub memory_info: Option<MemoryInfo>,
    pub cpu_info: Option<CpuInfo>,
    pub disks_info: Option<DisksInfo>,
    pub network_info: Option<NetworkInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDataForFrontend {
    pub id: String,
    pub pc_name: String,
    pub last_seen: String,
    pub online: bool,
    pub system_info: Option<SystemInfo>,
    pub memory_info: Option<MemoryInfo>,
    pub cpu_info: Option<CpuInfo>,
    pub disks_info: Option<DisksInfo>,
    pub network_info: Option<NetworkInfo>,
}

impl From<ServerData> for ServerDataForFrontend {
    fn from(server: ServerData) -> Self {
        Self {
            id: server.id,
            pc_name: server.pc_name,
            last_seen: server.last_seen,
            online: server.online,
            system_info: server.system_info,
            memory_info: server.memory_info,
            cpu_info: server.cpu_info,
            disks_info: server.disks_info,
            network_info: server.network_info,
        }
    }
}
