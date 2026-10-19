use crate::auth::{issue_token, AdminAuth};
use crate::data::DATA_MANAGER;
use crate::models::{
    CreateServerRequest, LoginRequest, RegistrationInfo, ServerDataForFrontend, SystemInfoPayload,
};
use crate::store::METRIC_STORE;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;

pub async fn api_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(json!({"error": "找不到此 API"}))
}

pub async fn get_servers(_: AdminAuth) -> Result<HttpResponse> {
    let servers = DATA_MANAGER.get_all_servers();

    let frontend_servers: HashMap<String, ServerDataForFrontend> = servers
        .into_iter()
        .map(|(id, server)| (id, server.into()))
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "servers": frontend_servers
    })))
}

pub async fn get_server(_: AdminAuth, path: web::Path<String>) -> Result<HttpResponse> {
    let server_id = path.into_inner();

    if let Some(server) = DATA_MANAGER.get_server(&server_id) {
        let frontend_server: ServerDataForFrontend = server.into();
        Ok(HttpResponse::Ok().json(frontend_server))
    } else {
        Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"})))
    }
}

pub async fn delete_server(_: AdminAuth, path: web::Path<String>) -> Result<HttpResponse> {
    let server_id = path.into_inner();

    match DATA_MANAGER.remove_server(&server_id) {
        Ok(_) => {
            if let Err(e) = METRIC_STORE.purge_server(&server_id) {
                log::error!("清除歷史資料失敗: {}", e);
            }
            Ok(HttpResponse::Ok().json(json!({"message": "伺服器已刪除"})))
        }
        Err(_) => Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"}))),
    }
}

pub async fn update_system_info(
    req: HttpRequest,
    payload: web::Json<SystemInfoPayload>,
) -> Result<HttpResponse> {
    let server_id = &payload.server_id;

    let Some(token) = bearer_token(&req) else {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "缺少憑證",
            "message": "請以 Authorization: Bearer <token> 標頭上報"
        })));
    };

    let Some(mut server) = DATA_MANAGER.verify_token(server_id, &token) else {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "憑證無效",
            "message": "伺服器 ID 或 token 不正確，請重新執行安裝指令"
        })));
    };

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    server.pc_name = payload.pc_info.pc_name.clone();
    server.ip_address = Some(client_ip);
    server.last_seen = chrono::Utc::now().to_rfc3339();
    server.online = true;
    server.system_info = Some(payload.system_info.clone());
    server.memory_info = Some(payload.memory_info.clone());
    server.cpu_info = Some(payload.cpu_info.clone());
    server.disks_info = Some(payload.disks_info.clone());
    server.network_info = Some(payload.network_info.clone());

    if let Err(e) = METRIC_STORE.record(&payload) {
        log::error!("寫入歷史資料失敗: {}", e);
    }

    match DATA_MANAGER.update_server(server_id, server) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status": "ok"}))),
        Err(e) => {
            log::error!("更新伺服器資料失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "更新伺服器資料失敗"
            })))
        }
    }
}

pub async fn get_stats(_: AdminAuth) -> Result<HttpResponse> {
    let stats = DATA_MANAGER.get_server_stats();
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn create_server(
    _: AdminAuth,
    req: HttpRequest,
    body: web::Json<CreateServerRequest>,
) -> Result<HttpResponse> {
    let name = body.name.trim();
    if name.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "名稱不可為空"})));
    }

    match DATA_MANAGER.create_server(name) {
        Ok(server) => {
            log::info!("已註冊伺服器 {} ({})", server.name, server.id);
            Ok(HttpResponse::Created().json(registration_info(&req, &server)))
        }
        Err(e) => {
            log::error!("建立伺服器失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "建立伺服器失敗"})))
        }
    }
}

pub async fn get_install_info(
    _: AdminAuth,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    match DATA_MANAGER.get_server(&path.into_inner()) {
        Some(server) => Ok(HttpResponse::Ok().json(registration_info(&req, &server))),
        None => Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"}))),
    }
}

pub async fn reset_token(
    _: AdminAuth,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let server_id = path.into_inner();
    match DATA_MANAGER.reset_token(&server_id) {
        Ok(server) => {
            log::info!("已重置伺服器 {} 的 token", server_id);
            Ok(HttpResponse::Ok().json(registration_info(&req, &server)))
        }
        Err(_) => Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"}))),
    }
}

pub async fn install_script() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/x-shellscript; charset=utf-8")
        .body(include_str!("install.sh")))
}

fn bearer_token(req: &HttpRequest) -> Option<String> {
    let raw = req.headers().get("Authorization")?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then(|| token.to_string())
}

fn panel_url(req: &HttpRequest) -> String {
    let info = req.connection_info().clone();
    format!("{}://{}", info.scheme(), info.host())
}

fn registration_info(req: &HttpRequest, server: &crate::models::ServerData) -> RegistrationInfo {
    let panel_url = panel_url(req);
    RegistrationInfo {
        install_command: format!(
            "curl -fsSL {url}/install.sh | sudo bash -s -- --url {url} --id {id} --token {token}",
            url = panel_url,
            id = server.id,
            token = server.token,
        ),
        id: server.id.clone(),
        name: server.name.clone(),
        token: server.token.clone(),
        panel_url,
    }
}

pub async fn login(body: web::Json<LoginRequest>) -> Result<HttpResponse> {
    if !DATA_MANAGER.verify_login(&body.username, &body.password) {
        log::warn!("登入失敗: {}", body.username);
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "帳號或密碼錯誤"})));
    }

    match issue_token(&body.username) {
        Ok((token, expires_at)) => Ok(HttpResponse::Ok().json(json!({
            "token": token,
            "expires_at": expires_at,
            "username": body.username,
        }))),
        Err(e) => {
            log::error!("簽發 token 失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "簽發 token 失敗"})))
        }
    }
}

pub async fn me(_: AdminAuth) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({"username": DATA_MANAGER.admin_username()})))
}

#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    pub range: Option<String>,
}

pub async fn get_history(
    _: AdminAuth,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> Result<HttpResponse> {
    let range = query.range.as_deref().unwrap_or("1h");
    let range_secs: i64 = match range {
        "1h" => 3600,
        "6h" => 6 * 3600,
        "24h" => 24 * 3600,
        "7d" => 7 * 86400,
        "30d" => 30 * 86400,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "range 必須是 1h / 6h / 24h / 7d / 30d"
            })))
        }
    };

    match METRIC_STORE.history(&path.into_inner(), range_secs) {
        Ok(points) => Ok(HttpResponse::Ok().json(json!({
            "range": range,
            "points": points,
        }))),
        Err(e) => {
            log::error!("查詢歷史資料失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "查詢歷史資料失敗"})))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct TrafficQuery {
    pub days: Option<i64>,
}

pub async fn get_traffic(
    _: AdminAuth,
    path: web::Path<String>,
    query: web::Query<TrafficQuery>,
) -> Result<HttpResponse> {
    let days = query.days.unwrap_or(30).clamp(1, 365);

    match METRIC_STORE.traffic(&path.into_inner(), days) {
        Ok(days_data) => {
            let rx: i64 = days_data.iter().map(|d| d.rx_bytes).sum();
            let tx: i64 = days_data.iter().map(|d| d.tx_bytes).sum();
            Ok(HttpResponse::Ok().json(json!({
                "days": days_data,
                "total_rx": rx,
                "total_tx": tx,
            })))
        }
        Err(e) => {
            log::error!("查詢流量失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "查詢流量失敗"})))
        }
    }
}

pub async fn get_diagnostics(_: AdminAuth) -> Result<HttpResponse> {
    match METRIC_STORE.diagnostics() {
        Ok(d) => Ok(HttpResponse::Ok().json(d)),
        Err(e) => {
            log::error!("取得診斷資訊失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "取得診斷資訊失敗"})))
        }
    }
}

pub async fn run_vacuum(_: AdminAuth) -> Result<HttpResponse> {
    match METRIC_STORE.vacuum() {
        Ok(freed) => {
            log::info!("手動空間回收完成，釋放 {} bytes", freed);
            Ok(HttpResponse::Ok().json(json!({"freed_bytes": freed})))
        }
        Err(e) => {
            log::error!("空間回收失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "空間回收失敗"})))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct RankingQuery {
    pub metric: Option<String>,
    pub limit: Option<usize>,
    pub days: Option<i64>,
}

pub async fn get_rankings(_: AdminAuth, query: web::Query<RankingQuery>) -> Result<HttpResponse> {
    let metric = query.metric.as_deref().unwrap_or("cpu");
    let limit = query.limit.unwrap_or(5).clamp(1, 100);
    let days = query.days.unwrap_or(30).clamp(1, 365);

    let servers = DATA_MANAGER.get_all_servers();

    let mut rows: Vec<serde_json::Value> = match metric {
        "rx" | "tx" | "traffic" => {
            let totals = match METRIC_STORE.traffic_totals(days) {
                Ok(t) => t,
                Err(e) => {
                    log::error!("查詢流量排行失敗: {}", e);
                    return Ok(HttpResponse::InternalServerError()
                        .json(json!({"error": "查詢流量排行失敗"})));
                }
            };
            totals
                .into_iter()
                .filter_map(|(id, rx, tx)| {
                    let server = servers.get(&id)?;
                    let value = match metric {
                        "rx" => rx as f64,
                        "tx" => tx as f64,
                        _ => (rx + tx) as f64,
                    };
                    Some(json!({
                        "id": id,
                        "name": server.name,
                        "online": server.online,
                        "value": value,
                        "rx_bytes": rx,
                        "tx_bytes": tx,
                    }))
                })
                .collect()
        }
        "cpu" | "mem" | "disk" => servers
            .values()
            .filter_map(|server| {
                let value = match metric {
                    "cpu" => {
                        let cpus = &server.cpu_info.as_ref()?.cpu_info;
                        if cpus.is_empty() {
                            return None;
                        }
                        cpus.iter().map(|c| c.cpu_usage as f64).sum::<f64>() / cpus.len() as f64
                    }
                    "mem" => {
                        let m = server.memory_info.as_ref()?;
                        if m.total_memory == 0 {
                            return None;
                        }
                        m.used_memory as f64 * 100.0 / m.total_memory as f64
                    }
                    _ => {
                        let disks = &server.disks_info.as_ref()?.disksinfo;
                        let total: u64 = disks.iter().map(|d| d.total_space).sum();
                        if total == 0 {
                            return None;
                        }
                        disks.iter().map(|d| d.used_space).sum::<u64>() as f64 * 100.0
                            / total as f64
                    }
                };
                Some(json!({
                    "id": server.id,
                    "name": server.name,
                    "online": server.online,
                    "value": value,
                }))
            })
            .collect(),
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "metric 必須是 cpu / mem / disk / rx / tx / traffic"
            })))
        }
    };

    rows.sort_by(|a, b| {
        b["value"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["value"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(limit);

    Ok(HttpResponse::Ok().json(json!({"metric": metric, "rankings": rows})))
}
