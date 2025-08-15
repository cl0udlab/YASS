use crate::models::ServerData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataStore {
    pub servers: HashMap<String, ServerData>,
    pub settings: AppSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub max_server_offline_time: u64, // 秒
    pub auto_cleanup_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_server_offline_time: 300, // 5分鐘
            auto_cleanup_enabled: true,
        }
    }
}

impl Default for DataStore {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            settings: AppSettings::default(),
        }
    }
}

pub struct JsonDataManager {
    data: Arc<Mutex<DataStore>>,
    file_path: String,
}

impl JsonDataManager {
    pub fn new(file_path: &str) -> Self {
        let manager = Self {
            data: Arc::new(Mutex::new(DataStore::default())),
            file_path: file_path.to_string(),
        };
        if let Err(e) = manager.load_from_file() {
            log::warn!("載入資料檔案失敗 {}: {}", file_path, e);
            log::info!("從空的資料儲存開始");
        }

        log::info!("重置伺服器資料");
        if let Err(e) = manager.clear_and_save() {
            log::warn!("重置伺服器資料時出錯: {}", e);
        }

        manager
    }

    pub fn load_from_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.file_path).exists() {
            log::info!(
                "Data file {} does not exist, will create on first save",
                self.file_path
            );
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path)?;
        let data: DataStore = serde_json::from_str(&content)?;

        let mut store = self.data.lock().unwrap();
        *store = data;

        log::info!(
            "Loaded {} servers from {}",
            store.servers.len(),
            self.file_path
        );
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.data.lock().unwrap();
        let content = serde_json::to_string_pretty(&*data)?;

        if let Some(parent) = Path::new(&self.file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.file_path, content)?;
        log::debug!("Saved data to {}", self.file_path);
        Ok(())
    }

    pub fn clear_and_save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();

        if data.servers.is_empty() {
            return Ok(());
        }

        for (_, server) in data.servers.iter_mut() {
            server.ip_address = None;
            server.last_seen = chrono::Utc::now().to_rfc3339();
            server.online = false;
            server.system_info = None;
            server.memory_info = None;
            server.cpu_info = None;
            server.disks_info = None;
            server.network_info = None;
        }

        drop(data);
        self.save_to_file()?;
        Ok(())
    }
    pub fn add_server(&self, server: ServerData) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        data.servers.insert(server.id.clone(), server);
        drop(data);
        self.save_to_file()
    }

    pub fn update_server(
        &self,
        server_id: &str,
        server: ServerData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        if data.servers.contains_key(server_id) {
            data.servers.insert(server_id.to_string(), server);
            drop(data);
            self.save_to_file()
        } else {
            Err("Server not found".into())
        }
    }

    pub fn remove_server(&self, server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        if data.servers.remove(server_id).is_some() {
            drop(data);
            self.save_to_file()
        } else {
            Err("Server not found".into())
        }
    }

    pub fn get_server(&self, server_id: &str) -> Option<ServerData> {
        let data = self.data.lock().unwrap();
        data.servers.get(server_id).cloned()
    }

    pub fn get_all_servers(&self) -> HashMap<String, ServerData> {
        let data = self.data.lock().unwrap();
        data.servers.clone()
    }

    pub fn server_exists(&self, server_id: &str) -> bool {
        let data = self.data.lock().unwrap();
        data.servers.contains_key(server_id)
    }

    pub fn get_settings(&self) -> AppSettings {
        let data = self.data.lock().unwrap();
        data.settings.clone()
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        data.settings = settings;
        drop(data);
        self.save_to_file()
    }

    pub fn get_server_stats(&self) -> ServerStats {
        let data = self.data.lock().unwrap();
        let total = data.servers.len();
        let online = data.servers.values().filter(|s| s.online).count();

        ServerStats {
            total_servers: total,
            online_servers: online,
            offline_servers: total - online,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStats {
    pub total_servers: usize,
    pub online_servers: usize,
    pub offline_servers: usize,
}

lazy_static::lazy_static! {
    pub static ref DATA_MANAGER: JsonDataManager = JsonDataManager::new("data/servers.json");
}
