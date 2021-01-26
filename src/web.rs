//! This module handles web routing and template rendering.

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use tera::{Context, Tera};

use crate::{Instance, PressResult};

#[get("/")]
async fn index(state: web::Data<State>) -> impl Responder {
    let posts = state.instance.load_posts(false);
    if let Err(_) = posts {
        return HttpResponse::InternalServerError().finish();
    }
    let posts = posts.unwrap();

    let mut context = Context::new();
    context.insert("entries", &posts);
    HttpResponse::Ok().body(state.templates.render("index.html", &context).unwrap())
}

#[get("/page/{page_num}/")]
async fn index_page(
    state: web::Data<State>,
    web::Path(page_num): web::Path<u32>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("page_num: {}", page_num))
}

#[get("/post/{year}/{month}/{day}/{name}/")]
async fn post(
    state: web::Data<State>,
    web::Path((year, month, day, name)): web::Path<(String, String, String, String)>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("post: {}-{}-{}-{}", year, month, day, name))
}

#[get("/archive/")]
async fn archive(state: web::Data<State>) -> impl Responder {
    HttpResponse::Ok().body("archive")
}

#[get("/category/{name}/")]
async fn category(state: web::Data<State>, web::Path(name): web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("category: {}", name))
}

#[get("/tag/{name}/")]
async fn tag(state: web::Data<State>, web::Path(name): web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("tag: {}", name))
}

#[get("/404.html")]
async fn page_not_found(state: web::Data<State>) -> impl Responder {
    HttpResponse::Ok().body("404")
}

struct State {
    instance: Instance,
    templates: Tera,
}

/// Serve Pressure instance as a web app.
pub fn serve(instance: Instance, host: &str, port: u16) -> PressResult<()> {
    let addr = format!("{}:{}", host, port);
    Ok(actix_web::rt::System::new("main").block_on(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(State {
                    instance: instance.clone(),
                    templates: Tera::new(
                        instance
                            .template_folder
                            .join("**")
                            .join("*.html")
                            .to_str()
                            .unwrap(),
                    )
                    .expect("Failed to parse templates."),
                }))
                .service(index)
                .service(index_page)
                .service(post)
                .service(archive)
                .service(category)
                .service(tag)
                .service(page_not_found)
        })
        .bind(addr)?
        .run()
        .await
    })?)
}
