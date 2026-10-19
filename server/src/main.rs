use actix_files::{Files, NamedFile};
use actix_web::dev::{fn_service, ServiceRequest, ServiceResponse};
use actix_web::{middleware, web, App, HttpServer};
use log::info;
use std::path::PathBuf;

const SWEEP_INTERVAL_SECS: u64 = 15;
const ROLLUP_INTERVAL_SECS: u64 = 300;

/// SvelteKit adapter-static 產生的 SPA 入口；任何不是實體檔案的路徑都回這個
const SPA_FALLBACK: &str = "200.html";

mod auth;
mod data;
mod handlers;
mod models;
mod store;

pub fn router_config(app: &mut actix_web::web::ServiceConfig) {
    app.route("/install.sh", web::get().to(handlers::install_script))
        .route("/download/client", web::get().to(handlers::download_client))
        .service(
            web::scope("/api")
                .route("/auth/login", web::post().to(handlers::login))
                .route("/auth/me", web::get().to(handlers::me))
                .route("/system_info", web::post().to(handlers::update_system_info))
                .route("/servers", web::get().to(handlers::get_servers))
                .route("/servers", web::post().to(handlers::create_server))
                .route("/servers/{id}", web::get().to(handlers::get_server))
                .route("/servers/{id}", web::delete().to(handlers::delete_server))
                .route(
                    "/servers/{id}/install",
                    web::get().to(handlers::get_install_info),
                )
                .route(
                    "/servers/{id}/reset_token",
                    web::post().to(handlers::reset_token),
                )
                .route("/stats", web::get().to(handlers::get_stats))
                .route(
                    "/servers/{id}/history",
                    web::get().to(handlers::get_history),
                )
                .route(
                    "/servers/{id}/traffic",
                    web::get().to(handlers::get_traffic),
                )
                .route("/rankings", web::get().to(handlers::get_rankings))
                .route("/diagnostics", web::get().to(handlers::get_diagnostics))
                .route("/maintenance/vacuum", web::post().to(handlers::run_vacuum))
                .default_service(web::to(handlers::api_not_found)),
        );
}

/// 前端建置產物：先找實體檔案，找不到就回 SPA 入口交給前端路由
fn web_service(web_dir: PathBuf) -> Files {
    let fallback = web_dir.join(SPA_FALLBACK);
    Files::new("/", web_dir)
        .index_file(SPA_FALLBACK)
        .default_handler(fn_service(move |req: ServiceRequest| {
            let fallback = fallback.clone();
            async move {
                let (req, _) = req.into_parts();
                let file = NamedFile::open_async(&fallback).await?;
                let res = file.into_response(&req);
                Ok(ServiceResponse::new(req, res))
            }
        }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let bind_addr = std::env::var("YASS_BIND").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    let web_dir =
        PathBuf::from(std::env::var("YASS_WEB_DIR").unwrap_or_else(|_| "web/build".to_string()));

    info!("Starting YASS server...");
    info!("Listening on http://{}", bind_addr);

    if web_dir.join(SPA_FALLBACK).is_file() {
        info!("前端目錄: {}", web_dir.display());
    } else {
        log::warn!(
            "找不到前端建置產物 {}，面板頁面將無法開啟（API 仍可用）。請先在 web/ 執行 pnpm build，或以 YASS_WEB_DIR 指定目錄",
            web_dir.join(SPA_FALLBACK).display()
        );
    }

    if !bind_addr.starts_with("127.") && !bind_addr.starts_with("localhost") {
        log::warn!(
            "面板正在對外監聽，但管理 API 目前尚無身分驗證：             任何能連到 {} 的人都可以新增或刪除伺服器。請先擋在反向代理或防火牆後面。",
            bind_addr
        );
    }

    actix_web::rt::spawn(async {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(SWEEP_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            match data::DATA_MANAGER.sweep_offline() {
                Ok(dropped) => {
                    for name in dropped {
                        log::warn!("伺服器 {} 逾時未回報，標記為離線", name);
                    }
                }
                Err(e) => log::error!("掃描離線伺服器失敗: {}", e),
            }
        }
    });

    actix_web::rt::spawn(async {
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(ROLLUP_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            if let Err(e) = store::METRIC_STORE.rollup() {
                log::error!("歷史資料分層失敗: {}", e);
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(router_config)
            .service(web_service(web_dir.clone()))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
