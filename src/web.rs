//! This module handles web routing and template rendering.

use std::{cmp::min, collections::HashMap, lazy::OnceCell, path::PathBuf};

use actix_service::Service;
use actix_web::{
    dev::ResourceMap, get, test::TestRequest, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use tera::{Context, Tera};
use yaml_rust::Yaml;

use crate::{Entry, EntryType, Instance, PressResult};

fn new_context(state: &web::Data<State>) -> Context {
    let mut ctx = Context::new();
    ctx.insert("site", &state.instance.site);
    // ctx.insert("config", &state.instance.config);
    ctx
}

impl Entry {
    fn generate_url(&mut self, req: &HttpRequest) {
        match self.etype {
            EntryType::Post => {
                let elems: Vec<&str> = self
                    .filepath
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .splitn(4, "-")
                    .collect();
                self.url = Some(req.url_for("post", elems).unwrap().path().to_string())
            }
            _ => {}
        }
    }
}

fn load_posts(instance: &Instance, req: &HttpRequest, meta_only: bool) -> Vec<Entry> {
    let mut posts = instance.load_posts(meta_only).unwrap();
    posts.iter_mut().for_each(|p| p.generate_url(req));
    return posts;
}

fn handle_index_page(state: web::Data<State>, req: HttpRequest, page_num: usize) -> impl Responder {
    let posts_per_page = state.instance.config.posts_per_index_page as usize;
    let mut posts = load_posts(&state.instance, &req, true);
    let post_count = posts.len();
    let page_count = (post_count + posts_per_page - 1) / posts_per_page;
    if page_num < 1 || page_num > page_count {
        return HttpResponse::NotFound().finish();
    }
    let prev_url = if page_num == 1 {
        "".to_string()
    } else if page_num == 2 {
        req.url_for_static("index").unwrap().path().to_string()
    } else {
        req.url_for("index_page", &[(page_num - 1).to_string()])
            .unwrap()
            .path()
            .to_string()
    };
    let next_url = if page_num < page_count {
        req.url_for("index_page", &[(page_num + 1).to_string()])
            .unwrap()
            .path()
            .to_string()
    } else {
        "".to_string()
    };
    let begin = (page_num - 1) * posts_per_page;
    let end = min(post_count, begin + posts_per_page);
    let posts_to_render = &mut posts[begin..end];
    posts_to_render.iter_mut().for_each(|p| {
        p.load_content();
    });

    let mut context = new_context(&state);
    context.insert("entries", posts_to_render);
    context.insert(
        "pager",
        &hashmap! {"prev_url" => prev_url, "next_url" => next_url},
    );
    HttpResponse::Ok().body(state.templates.render("index.html", &context).unwrap())
}

#[get("/")]
async fn index(state: web::Data<State>, req: HttpRequest) -> impl Responder {
    handle_index_page(state, req, 1)
}

#[get("/page/{page_num}/")]
async fn index_page(
    state: web::Data<State>,
    req: HttpRequest,
    web::Path(page_num): web::Path<usize>,
) -> impl Responder {
    handle_index_page(state, req, page_num)
}

#[get(r#"/post/{year:\d{4}}/{month:\d{2}}/{day:\d{2}}/{name}/"#)]
async fn post(
    state: web::Data<State>,
    web::Path((year, month, day, name)): web::Path<(u16, u8, u8, String)>,
) -> impl Responder {
    let post = state.instance.load_post(year, month, day, &name, false);
    if post.is_err() {
        return HttpResponse::NotFound().finish();
    }
    let post = post.unwrap();
    let mut context = new_context(&state);
    context.insert("entry", &post);
    HttpResponse::Ok().body(state.templates.render("post.html", &context).unwrap())
}

#[get("/archive/")]
async fn archive(state: web::Data<State>, req: HttpRequest) -> impl Responder {
    let posts = load_posts(&state.instance, &req, true);
    let mut context = new_context(&state);
    context.insert("entries", &posts);
    context.insert("archive", &hashmap! {"type" => "Archive", "name" => "All"});
    HttpResponse::Ok().body(state.templates.render("archive.html", &context).unwrap())
}

#[get("/category/{name}/")]
async fn category(
    state: web::Data<State>,
    req: HttpRequest,
    web::Path(name): web::Path<String>,
) -> impl Responder {
    let posts = load_posts(&state.instance, &req, true);
    let mut context = new_context(&state);
    context.insert(
        "entries",
        &posts
            .iter()
            .filter(|p| {
                p.meta["categories"]
                    .as_vec()
                    .unwrap()
                    .contains(&Yaml::String(name.clone()))
            })
            .collect::<Vec<&Entry>>(),
    );
    context.insert("archive", &hashmap! {"type" => "Category", "name" => &name});
    HttpResponse::Ok().body(state.templates.render("archive.html", &context).unwrap())
}

#[get("/tag/{name}/")]
async fn tag(
    state: web::Data<State>,
    req: HttpRequest,
    web::Path(name): web::Path<String>,
) -> impl Responder {
    let posts = load_posts(&state.instance, &req, true);
    let mut context = new_context(&state);
    context.insert(
        "entries",
        &posts
            .iter()
            .filter(|p| {
                p.meta["tags"]
                    .as_vec()
                    .unwrap()
                    .contains(&Yaml::String(name.clone()))
            })
            .collect::<Vec<&Entry>>(),
    );
    context.insert("archive", &hashmap! {"type" => "Tag", "name" => &name});
    HttpResponse::Ok().body(state.templates.render("archive.html", &context).unwrap())
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

#[get("/{rel_url:.*}")]
async fn page(
    state: web::Data<State>,
    req: HttpRequest,
    web::Path(rel_url): web::Path<PathBuf>,
) -> impl Responder {
    if let Ok(page) = state.instance.load_page(&rel_url) {
        let mut context = new_context(&state);
        context.insert("entry", &page);
        HttpResponse::Ok().body(state.templates.render("page.html", &context).unwrap())
    } else {
        let filepath = state.instance.raw_folder.join(&rel_url);
        if !filepath.starts_with(&state.instance.raw_folder) {
            return HttpResponse::Forbidden().finish();
        }
        if let Ok(file) = actix_files::NamedFile::open(filepath) {
            file.into_response(&req).unwrap()
        } else {
            HttpResponse::NotFound().finish()
        }
    }
}

thread_local! {
    static ROUTES_KEY: OnceCell<ResourceMap> = OnceCell::new();
}

fn tera_url_for(args: &HashMap<String, tera::Value>) -> Result<tera::Value, tera::Error> {
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
        Ok(tera::Value::String(url.path().to_string())) // TODO: prepend url root
    })
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

            tera.register_function("url_for", tera_url_for);

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
                .service(page)
        })
        .bind(addr)?
        .run()
        .await
    })?)
}
