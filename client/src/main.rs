mod report;
mod system;

use report::{print_system_info,print_memory_info};
use sysinfo::System;
fn main() {
    let mut system = System::new();
    system.refresh_all();
    let system_info = system::get_system_info(&system);
    print_system_info(&system_info);
    let memory_info = system::get_memory_info(&system);
    print_memory_info(&memory_info);
}
