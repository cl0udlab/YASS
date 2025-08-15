use crate::data::DATA_MANAGER;
use crate::jinja::render_spa;
use crate::models::{ServerData, ServerDataForFrontend, SystemInfoPayload};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;

pub async fn spa_handler() -> Result<HttpResponse> {
    match render_spa() {
        Ok(html) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)),
        Err(e) => {
            eprintln!("渲染頁面失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "渲染頁面失敗"})))
        }
    }
}

pub async fn get_servers() -> Result<HttpResponse> {
    let servers = DATA_MANAGER.get_all_servers();

    let frontend_servers: HashMap<String, ServerDataForFrontend> = servers
        .into_iter()
        .map(|(id, server)| (id, server.into()))
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "servers": frontend_servers
    })))
}

pub async fn get_server(path: web::Path<String>) -> Result<HttpResponse> {
    let server_id = path.into_inner();

    if let Some(server) = DATA_MANAGER.get_server(&server_id) {
        let frontend_server: ServerDataForFrontend = server.into();
        Ok(HttpResponse::Ok().json(frontend_server))
    } else {
        Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"})))
    }
}

pub async fn delete_server(path: web::Path<String>) -> Result<HttpResponse> {
    let server_id = path.into_inner();

    match DATA_MANAGER.remove_server(&server_id) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"message": "伺服器已刪除"}))),
        Err(_) => Ok(HttpResponse::NotFound().json(json!({"error": "伺服器未找到"}))),
    }
}

pub async fn update_system_info(
    req: HttpRequest,
    payload: web::Json<SystemInfoPayload>,
) -> Result<HttpResponse> {
    let server_id = &payload.server_id;

    if !DATA_MANAGER.server_exists(server_id) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "未授權的伺服器 ID",
            "message": "此伺服器 ID 尚未在系統中註冊"
        })));
    }

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    let server_data = ServerData {
        id: server_id.clone(),
        pc_name: payload.pc_info.pc_name.clone(),
        ip_address: Some(client_ip),
        last_seen: chrono::Utc::now().to_rfc3339(),
        online: true,
        system_info: Some(payload.system_info.clone()),
        memory_info: Some(payload.memory_info.clone()),
        cpu_info: Some(payload.cpu_info.clone()),
        disks_info: Some(payload.disks_info.clone()),
        network_info: Some(payload.network_info.clone()),
    };

    match DATA_MANAGER.update_server(server_id, server_data) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"status": "ok"}))),
        Err(e) => {
            log::error!("更新伺服器資料失敗: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "更新伺服器資料失敗"
            })))
        }
    }
}

pub async fn get_stats() -> Result<HttpResponse> {
    let stats = DATA_MANAGER.get_server_stats();
    Ok(HttpResponse::Ok().json(stats))
}
