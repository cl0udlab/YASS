mod report;
mod system;

use anyhow::{bail, Result};
use report::print_pc_info;
use serde::Serialize;
use std::time::Duration;
use sysinfo::System;
use tokio::time;

#[derive(Serialize)]
struct SystemInfoPayload {
    pc_info: system::PCInfo,
    system_info: system::SystemInfo,
    memory_info: system::MemoryInfo,
    network_info: system::NetworkInfo,
    disks_info: system::DisksInfo,
    cpu_info: system::CpuInfo,
}

const API_URL: &str = "http://127.0.0.1:9000/api/system_info";

const UPDATE_INTERVAL_SECS: u64 = 60;

#[tokio::main]
async fn main() -> Result<()> {
    println!("系統監控客戶端已啟動");
    println!("將每 {} 秒向 API 發送一次資料", UPDATE_INTERVAL_SECS);

    let mut interval = time::interval(Duration::from_secs(UPDATE_INTERVAL_SECS));

    loop {
        interval.tick().await;

        let mut system = System::new_all();
        system.refresh_all();

        let pc_info = system::get_pc_info();
        let system_info = system::get_system_info(&system);
        let memory_info = system::get_memory_info(&system);
        let disks_info = system::get_disks_info();
        let network_info = system::get_network_info();
        let cpu_info = system::get_cpu_info(&system);

        println!("正在收集系統資訊...");
        print_pc_info(&pc_info);

        let payload = SystemInfoPayload {
            pc_info,
            system_info,
            memory_info,
            network_info,
            disks_info,
            cpu_info,
        };

        // 發送資料到 API
        match send_system_info(&payload).await {
            Ok(_) => println!("系統資訊已成功發送到 API"),
            Err(e) => eprintln!("發送系統資訊時出錯: {}", e),
        }
    }
}

async fn send_system_info(payload: &SystemInfoPayload) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.post(API_URL).json(payload).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        bail!("API 返回錯誤 ({}): {}", status, error_text)
    }
}
