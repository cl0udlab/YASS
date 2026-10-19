mod report;
mod system;

use anyhow::{bail, Context, Result};
use report::print_pc_info;
use serde::Serialize;
use std::time::Duration;
use sysinfo::System;
use tokio::time;

#[derive(Serialize)]
struct SystemInfoPayload {
    server_id: String,
    pc_info: system::PCInfo,
    system_info: system::SystemInfo,
    memory_info: system::MemoryInfo,
    network_info: system::NetworkInfo,
    disks_info: system::DisksInfo,
    cpu_info: system::CpuInfo,
}

struct Config {
    api_url: String,
    server_id: String,
    token: String,
    interval: Duration,
}

impl Config {
    fn from_env() -> Result<Self> {
        let panel_url = std::env::var("YASS_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string())
            .trim_end_matches('/')
            .to_string();

        let server_id = std::env::var("YASS_SERVER_ID")
            .context("缺少 YASS_SERVER_ID，請在面板上新增伺服器並執行產生的安裝指令")?;
        let token = std::env::var("YASS_TOKEN")
            .context("缺少 YASS_TOKEN，請在面板上新增伺服器並執行產生的安裝指令")?;

        let interval = std::env::var("YASS_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60)
            .max(1);

        Ok(Self {
            api_url: format!("{}/api/system_info", panel_url),
            server_id,
            token,
            interval: Duration::from_secs(interval),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    println!("系統監控客戶端已啟動");
    println!("回報目標: {}", config.api_url);
    println!("伺服器 ID: {}", config.server_id);
    println!("回報間隔: {} 秒", config.interval.as_secs());

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut system = System::new_all();

    let mut interval = time::interval(config.interval);

    loop {
        interval.tick().await;

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
            server_id: config.server_id.clone(),
            pc_info,
            system_info,
            memory_info,
            network_info,
            disks_info,
            cpu_info,
        };

        match send_system_info(&http, &config, &payload).await {
            Ok(_) => println!("系統資訊已成功發送到 API"),
            Err(e) => eprintln!("發送系統資訊時出錯: {}", e),
        }
    }
}

async fn send_system_info(
    http: &reqwest::Client,
    config: &Config,
    payload: &SystemInfoPayload,
) -> Result<()> {
    let response = http
        .post(&config.api_url)
        .bearer_auth(&config.token)
        .json(payload)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        bail!(
            "憑證被拒（{}）：{}。面板可能已重置 token 或刪除此伺服器，請重新執行安裝指令。",
            status,
            body
        );
    }
    bail!("API 返回錯誤 ({}): {}", status, body)
}
