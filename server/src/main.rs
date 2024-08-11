use actix_web::{web, HttpResponse, HttpServer};

pub fn router_config(app: &mut actix_web::web::ServiceConfig) {
    app.service(web::scope("/api").route("/hello", web::get().to(|| HttpResponse::Ok())));
}

#[actix_web::main]
async fn main() {
    HttpServer::new(move || actix_web::App::new().configure(crate::router_config).route("/", web::get().to(|| HttpResponse::Ok())))
        .bind("127.0.0.1:9000")
        .expect("Can not bind to port 9000")
        .run()
        .await
        .expect("Server crashed");
}
