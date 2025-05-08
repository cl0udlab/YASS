use os_info;
use sysinfo::{Disks, Networks, System};
use serde::Serialize;

#[derive(Serialize)]
pub struct PCInfo {
    pub pc_name: String,
}
#[derive(Serialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_name: String,
    pub uptime: u64,
    pub process_count: u64,
}
#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}
#[derive(Serialize)]
pub struct Network {
    pub interface_name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}
#[derive(Serialize)]
pub struct NetworkInfo {
    pub network_info: Vec<Network>,
}
#[derive(Serialize)]
pub struct Disk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
}
#[derive(Serialize)]
pub struct DisksInfo {
    pub disksinfo: Vec<Disk>,
}
#[derive(Serialize)]
pub struct OneCpuInfo {
    pub cpu_name: String,
    pub cpu_usage: f32,
}
#[derive(Serialize)]
pub struct CpuInfo {
    pub cpu_info: Vec<OneCpuInfo>,
}

pub fn get_network_info() -> NetworkInfo {
    let mut networks = Networks::new();
    networks.refresh_list();

    let mut networkinfo = NetworkInfo {
        network_info: Vec::new(),
    };
    for (interface_name, network) in networks.iter() {
        networkinfo.network_info.push(Network {
            interface_name: interface_name.to_string(),
            received_bytes: network.total_received(),
            transmitted_bytes: network.total_transmitted(),
        });
    }
    networkinfo
}

pub fn get_cpu_info(s: &System) -> CpuInfo {
    let mut cpuinfo = CpuInfo {
        cpu_info: Vec::new(),
    };

    for (i, cpu) in s.cpus().iter().enumerate() {
        cpuinfo.cpu_info.push(OneCpuInfo {
            cpu_name: format!("CPU{}", i),
            cpu_usage: cpu.cpu_usage(),
        });
    }
    cpuinfo
}

pub fn get_disks_info() -> DisksInfo {
    let disks = Disks::new_with_refreshed_list();
    let mut diskinfo = DisksInfo {
        disksinfo: Vec::new(),
    };
    for disk in disks.iter() {
        diskinfo.disksinfo.push(Disk {
            name: disk.name().to_str().unwrap().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            used_space: disk.total_space() - disk.available_space(),
            file_system: disk.file_system().to_str().unwrap().to_string(),
        });
    }
    diskinfo
}

pub fn get_pc_info() -> PCInfo {
    PCInfo {
        pc_name: hostname::get().unwrap().to_string_lossy().to_string(),
    }
}

pub fn get_system_info(s: &System) -> SystemInfo {
    let cpu_name = s.cpus()[0].brand().to_string();
    SystemInfo {
        os_name: os_info::get().os_type().to_string(),
        os_version: os_info::get().version().to_string(),
        kernel_version: System::kernel_version().unwrap_or("Unknown".to_string()),
        cpu_name,
        uptime: System::uptime(),
        process_count: s.processes().len() as u64,
    }
}

pub fn get_memory_info(s: &System) -> MemoryInfo {
    MemoryInfo {
        total_memory: s.total_memory(),
        used_memory: s.used_memory(),
        total_swap: s.total_swap(),
        used_swap: s.used_swap(),
    }
}
