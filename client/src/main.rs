mod report;
mod system;

use report::{print_system_info,print_memory_info,print_pc_info,print_disks_info,print_network_info};
use sysinfo::System;
fn main() {
    let mut system = System::new_all();
    system.refresh_all();
    let system_info = system::get_system_info(&system);
    print_system_info(&system_info);
    let memory_info = system::get_memory_info(&system);
    print_memory_info(&memory_info);
    let pc_info = system::get_pc_info();
    print_pc_info(&pc_info);
    let disk_info = system::get_disks_info();
    print_disks_info(&disk_info);
    let networkinfo = system::get_network_info();
    print_network_info(&networkinfo);
}
