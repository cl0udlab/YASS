use crate::system::{SystemInfo, MemoryInfo, PCInfo, DisksInfo};

pub fn print_system_info(system_info: &SystemInfo) {
    println!("OS Name: {}", system_info.os_name);
    println!("OS Version: {}", system_info.os_version);
    println!("Kernel Version: {}", system_info.kernel_version);
    println!("Uptime: {} seconds", system_info.uptime);
    println!("CPU Name: {:?}", system_info.cpu_name);
    println!("Uptime: {} {} {} {}", system_info.uptime / 86400, system_info.uptime / 3600 % 24, system_info.uptime / 60 % 60, system_info.uptime % 60);
}

pub fn print_memory_info(memory_info: &MemoryInfo) {
    println!("Total Memory: {} GB", memory_info.total_memory / 1024 / 1024 / 1024);
    println!("Used Memory: {} GB", memory_info.used_memory / 1024 / 1024 / 1024);
    println!("Total Swap: {} GB", memory_info.total_swap / 1024 / 1024 / 1024);
    println!("Used Swap: {} GB", memory_info.used_swap / 1024 / 1024 / 1024);
}

pub fn print_pc_info(pc_info: &PCInfo) {
    println!("PC Name: {}", pc_info.pc_name);
}

pub fn print_disks_info(disks_info: &DisksInfo) {
    for disk in disks_info.disksinfo.iter() {
        println!("Disk Name: {}", disk.name);
        println!("Total Space: {} GB", disk.total_space / 1024 / 1024 / 1024);
        println!("Available Space: {} GB", disk.available_space / 1024 / 1024 / 1024);
        println!("Used Space: {} GB", disk.used_space / 1024 / 1024 / 1024);
        println!("File System: {}", disk.file_system);
    }
}