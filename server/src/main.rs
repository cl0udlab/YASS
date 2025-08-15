use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use log::info;

mod data;
mod handlers;
mod jinja;
mod models;

pub fn router_config(app: &mut actix_web::web::ServiceConfig) {
    app.route("/", web::get().to(handlers::spa_handler))
        .service(
            web::scope("/api")
                .route("/system_info", web::post().to(handlers::update_system_info))
                .route("/servers", web::get().to(handlers::get_servers))
                .route("/servers/{id}", web::get().to(handlers::get_server))
                .route("/servers/{id}", web::delete().to(handlers::delete_server))
                .route("/stats", web::get().to(handlers::get_stats)),
        );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Starting YASS server...");
    info!("Web interface available at: http://127.0.0.1:9000");
    info!("API endpoint: http://127.0.0.1:9000/api/system_info");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(router_config)
            .default_service(
                web::route().to(|| async { HttpResponse::NotFound().body("Page not found") }),
            )
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}
