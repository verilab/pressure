#[macro_use]
extern crate lazy_static;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        TEMPLATES
            .render("index.html", &tera::Context::new())
            .unwrap(),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
