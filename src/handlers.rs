use actix_web::{get, web, HttpResponse, Responder};
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*.html") {
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
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        TEMPLATES
            .render("index.html", &tera::Context::new())
            .unwrap(),
    )
}

#[get("/page/{page_num}/")]
pub async fn index_page(web::Path(page_num): web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!("page_num: {}", page_num))
}

#[get("/post/{year}/{month}/{day}/{name}/")]
pub async fn post(
    web::Path((year, month, day, name)): web::Path<(String, String, String, String)>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("post: {}-{}-{}-{}", year, month, day, name))
}

#[get("/archive/")]
pub async fn archive() -> impl Responder {
    HttpResponse::Ok().body("archive")
}

#[get("/category/{name}/")]
pub async fn category(web::Path(name): web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("category: {}", name))
}

#[get("/tag/{name}/")]
pub async fn tag(web::Path(name): web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("tag: {}", name))
}

#[get("/404.html")]
pub async fn page_not_found() -> impl Responder {
    HttpResponse::Ok().body("404")
}
