use actix_web::{web, HttpResponse, HttpServer};
use log::info;
mod models;
mod handlers;

pub fn router_config(app: &mut actix_web::web::ServiceConfig) {
    app.service(
        web::scope("/api")
            .route("/hello", web::get().to(|| HttpResponse::Ok()))
            .route("/system_info", web::post().to(handlers::receive_system_info))
    );
}

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting server...");
    info!("Listening on 127.0.0.1:9000");

    HttpServer::new(move || {
        actix_web::App::new()
            .configure(crate::router_config)
            .route("/", web::get().to(|| HttpResponse::Ok()))
    })
    .bind("127.0.0.1:9000")
    .expect("Can not bind to port 9000")
    .run()
    .await
    .expect("Server crashed");
}