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
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub username: String,
    pub password_hash: String,
    pub jwt_secret: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            password_hash: String::new(),
            jwt_secret: String::new(),
        }
    }
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
            auth: AuthConfig::default(),
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

        if let Err(e) = manager.ensure_auth() {
            log::error!("初始化管理員帳號失敗: {}", e);
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

    pub fn create_server(&self, name: &str) -> Result<ServerData, Box<dyn std::error::Error>> {
        let now = chrono::Utc::now().to_rfc3339();
        let server = ServerData {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            token: generate_token(),
            created_at: now.clone(),
            pc_name: name.to_string(),
            ip_address: None,
            last_seen: now,
            online: false,
            system_info: None,
            memory_info: None,
            cpu_info: None,
            disks_info: None,
            network_info: None,
        };

        let mut data = self.data.lock().unwrap();
        data.servers.insert(server.id.clone(), server.clone());
        drop(data);
        self.save_to_file()?;
        Ok(server)
    }

    pub fn reset_token(&self, server_id: &str) -> Result<ServerData, Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        let server = data.servers.get_mut(server_id).ok_or("Server not found")?;
        server.token = generate_token();
        let updated = server.clone();
        drop(data);
        self.save_to_file()?;
        Ok(updated)
    }

    pub fn verify_token(&self, server_id: &str, token: &str) -> Option<ServerData> {
        let data = self.data.lock().unwrap();
        let server = data.servers.get(server_id)?;
        if constant_time_eq(server.token.as_bytes(), token.as_bytes()) {
            Some(server.clone())
        } else {
            None
        }
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

    fn ensure_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();

        if data.auth.jwt_secret.is_empty() {
            data.auth.jwt_secret = generate_token();
        }

        if !data.auth.username.is_empty() && !data.auth.password_hash.is_empty() {
            drop(data);
            return self.save_to_file();
        }

        let username = std::env::var("YASS_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
        let (password, generated) = match std::env::var("YASS_ADMIN_PASSWORD") {
            Ok(p) if !p.is_empty() => (p, false),
            _ => (uuid::Uuid::new_v4().simple().to_string(), true),
        };

        data.auth.username = username.clone();
        data.auth.password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)?;
        drop(data);
        self.save_to_file()?;

        if generated {
            log::warn!("========================================");
            log::warn!("已建立管理員帳號");
            log::warn!("  帳號: {}", username);
            log::warn!("  密碼: {}", password);
            log::warn!("此密碼只會顯示這一次，請立刻保存");
            log::warn!("========================================");
        } else {
            log::info!("已用 YASS_ADMIN_PASSWORD 建立管理員帳號 {}", username);
        }

        Ok(())
    }

    pub fn verify_login(&self, username: &str, password: &str) -> bool {
        let data = self.data.lock().unwrap();
        if !constant_time_eq(data.auth.username.as_bytes(), username.as_bytes()) {
            return false;
        }
        bcrypt::verify(password, &data.auth.password_hash).unwrap_or(false)
    }

    pub fn jwt_secret(&self) -> String {
        self.data.lock().unwrap().auth.jwt_secret.clone()
    }

    pub fn admin_username(&self) -> String {
        self.data.lock().unwrap().auth.username.clone()
    }

    pub fn sweep_offline(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut data = self.data.lock().unwrap();
        let threshold = data.settings.max_server_offline_time as i64;
        let now = chrono::Utc::now();
        let mut dropped = Vec::new();

        for server in data.servers.values_mut() {
            if !server.online {
                continue;
            }

            let elapsed = match chrono::DateTime::parse_from_rfc3339(&server.last_seen) {
                Ok(seen) => (now - seen.with_timezone(&chrono::Utc)).num_seconds(),
                Err(_) => i64::MAX,
            };

            if elapsed > threshold {
                server.online = false;
                dropped.push(server.name.clone());
            }
        }

        drop(data);

        if !dropped.is_empty() {
            self.save_to_file()?;
        }
        Ok(dropped)
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

fn generate_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
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
