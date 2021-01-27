//! This module handles web routing and template rendering.

use std::{collections::HashMap, lazy::OnceCell, path::PathBuf};

use actix_service::Service;
use actix_web::{
    dev::ResourceMap, get, test::TestRequest, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use tera::{Context, Tera};

use crate::{Instance, PressResult};

fn new_context(state: &web::Data<State>) -> Context {
    let mut ctx = Context::new();
    ctx.insert("site", &state.instance.config.site);
    ctx
}

#[get("/")]
async fn index(state: web::Data<State>, req: HttpRequest) -> impl Responder {
    let posts = state.instance.load_posts(false);
    if posts.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    let posts = posts.unwrap();

    let rmap = req.resource_map().clone();
    println!("{:?}", rmap.url_for(&req, "index_page", &["2"]));

    // println!("{:?}", req.url_for("index_page", &["2"]));
    let mut context = new_context(&state);
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

#[get("/static/{filename:.*}", name = "static")]
async fn root_static(
    state: web::Data<State>,
    web::Path(filename): web::Path<PathBuf>,
) -> Result<actix_files::NamedFile, actix_web::Error> {
    Ok(actix_files::NamedFile::open(
        state.instance.static_folder.join(filename),
    )?)
}

#[get("/theme/static/{filename:.*}", name = "theme.static")]
async fn theme_static(
    state: web::Data<State>,
    web::Path(filename): web::Path<PathBuf>,
) -> Result<actix_files::NamedFile, actix_web::Error> {
    Ok(actix_files::NamedFile::open(
        state.instance.theme_static_folder.join(filename),
    )?)
}

thread_local! {
    static ROUTES_KEY: OnceCell<ResourceMap> = OnceCell::new();
}

fn make_url_for() -> impl tera::Function {
    move |args: &HashMap<String, tera::Value>| -> Result<tera::Value, tera::Error> {
        println!("args: {:?}", args);
        let name = args["name"]
            .as_str()
            .ok_or(tera::Error::msg("`name` should be a string"))?;
        let empty_elements = tera::Value::Array(vec![]);
        let elements_iter = args
            .get("elements")
            .unwrap_or(&empty_elements)
            .as_array()
            .ok_or(tera::Error::msg("`elements` should be an array"))?
            .iter();
        let mut elements = vec![];
        for elem in elements_iter {
            elements.push(elem.as_str().ok_or(tera::Error::msg(
                "`elements` array should contain only strings",
            ))?);
        }
        ROUTES_KEY.with(|routes| {
            let routes = routes.get().ok_or(tera::Error::msg(
                "`url_for` should only be called in request context",
            ))?;
            let fake_req = TestRequest::default().to_http_request();
            let url = routes
                .url_for(&fake_req, name, elements)
                .or(Err(tera::Error::msg("resource not found")))?;
            println!("url: {:?}", url);
            Ok(tera::Value::String(url.path().to_string()))
        })
    }
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
            let mut tera = Tera::new(
                instance
                    .template_folder
                    .join("**")
                    .join("*.html")
                    .to_str()
                    .unwrap(),
            )
            .expect("Failed to parse templates.");

            // let routes = Arc::new(Mutex::new(OnceCell::new()));
            tera.register_function("url_for", make_url_for());

            App::new()
                .app_data(web::Data::new(State {
                    instance: instance.clone(),
                    templates: tera,
                }))
                .wrap_fn(move |req, srv| {
                    ROUTES_KEY.with(|routes| {
                        routes.get_or_init(|| req.resource_map().clone());
                    });
                    srv.call(req)
                })
                .service(index)
                .service(index_page)
                .service(post)
                .service(archive)
                .service(category)
                .service(tag)
                .service(root_static)
                .service(theme_static)
        })
        .bind(addr)?
        .run()
        .await
    })?)
}
