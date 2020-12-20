use actix_web::{App, HttpServer};

#[macro_use]
extern crate lazy_static;

mod handlers;

pub fn serve(host: &str, port: u16) -> std::io::Result<()> {
    let addr = format!("{}:{}", host, port);
    actix_web::rt::System::new("main").block_on(async move {
        HttpServer::new(|| {
            App::new()
                .service(handlers::index)
                .service(handlers::index_page)
                .service(handlers::post)
                .service(handlers::archive)
                .service(handlers::category)
                .service(handlers::tag)
                .service(handlers::page_not_found)
        })
        .bind(addr)?
        .run()
        .await
    })
}
