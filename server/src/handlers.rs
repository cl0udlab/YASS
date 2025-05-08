use actix_web::{web, HttpResponse, Result};
use log::info;

use crate::models::SystemInfoPayload;

pub async fn receive_system_info(payload: web::Json<SystemInfoPayload>) -> Result<HttpResponse> {
    let info = payload.into_inner();

    info!("Received system info from: {}", info.pc_info.pc_name);

    info!(
        "OS: {} {}",
        info.system_info.os_name, info.system_info.os_version
    );
    info!(
        "Memory usage: {}/{} MB",
        info.memory_info.used_memory / 1024 / 1024,
        info.memory_info.total_memory / 1024 / 1024
    );

    //TODO: DB

    Ok(HttpResponse::Ok().json(web::Json(serde_json::json!({
        "status": "success",
        "message": "System information received and processed"
    }))))
}
