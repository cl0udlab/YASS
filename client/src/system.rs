use os_info;
use sysinfo::{Components, Cpu, Disks, Networks, System};

pub struct PCInfo {
    pub pc_name: String,
}
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_name: String,
    pub uptime: u64,
}

pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

pub struct NetworkInfo {
    pub interface_name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

pub struct DiskInfo {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
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
