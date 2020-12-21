use std::path::PathBuf;

use actix_web::{App, HttpServer};
use tera::Tera;

mod handlers;

#[derive(Debug)]
pub struct Pressure {
    instance_folder: PathBuf,
    static_folder: PathBuf,
    template_folder: PathBuf,
    theme_static_folder: PathBuf,
    posts_folder: PathBuf,
    pages_folder: PathBuf,
    raw_folder: PathBuf,
    tera: Tera,
}

impl Pressure {
    pub fn new<T: Into<PathBuf>>(instance_folder: T) -> Pressure {
        let instance_folder = instance_folder.into();
        let static_folder = instance_folder.join("static");
        let template_folder = instance_folder.join("theme").join("templates");
        let theme_static_folder = instance_folder.join("theme").join("static");
        let posts_folder = instance_folder.join("posts");
        let pages_folder = instance_folder.join("pages");
        let raw_folder = instance_folder.join("raw");
        let tera =
            Tera::new(template_folder.to_str().unwrap()).expect("Failed to parse templates.");
        Pressure {
            instance_folder,
            static_folder,
            template_folder,
            theme_static_folder,
            posts_folder,
            pages_folder,
            raw_folder,
            tera,
        }
    }

    pub fn serve(&self, host: &str, port: u16) -> std::io::Result<()> {
        let addr = format!("{}:{}", host, port);
        actix_web::rt::System::new("main").block_on(async move {
            HttpServer::new(|| {
                App::new()
                // .service(handlers::index)
                // .service(handlers::index_page)
                // .service(handlers::post)
                // .service(handlers::archive)
                // .service(handlers::category)
                // .service(handlers::tag)
                // .service(handlers::page_not_found)
            })
            .bind(addr)?
            .run()
            .await
        })
    }
}
