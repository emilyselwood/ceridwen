use std::collections::HashSet;
use std::env;
use std::io;
use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::get;
use actix_web::http::header::ContentType;
use actix_web::middleware::Logger;
use actix_web::post;
use actix_web::web;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use ceridwen::config::Config;
use ceridwen::index_sled::Index;
use ceridwen::search_result::SearchResult;
use env_logger::Env;
use log::debug;
use log::info;
use log::warn;
use serde::Deserialize;
use tera::Context;
use tera::Tera;

use crate::error::Error;

mod error;

pub struct AppData {
    _config: Config,
    templates: Tera,
}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    let result = run_server().await;
    if result.is_err() {
        let err = result.unwrap_err();
        println!("Error running server! {}", err);
        return Err(err);
    }

    Ok(())
}

async fn run_server() -> Result<(), Error> {
    println!("Server starting. Loading config");
    // load config
    let config = Config::load()?;
    println!("Config loaded. Setting up logging");
    // Set up logging
    env_logger::init_from_env(Env::default().default_filter_or(config.server.log_level.clone()));
    info!(
        "Logging setup. Logging level set to {}",
        config.server.log_level.clone(),
    );

    let app_data = AppData {
        _config: config.clone(),
        templates: load_templates()?,
    };

    let web_data = web::Data::new(app_data);

    // set up server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web_data.clone())
            .service(post_search)
            .service(get_search)
            // General file routes. images, css, and javascript
            .route(
                "/img/{filename:.*\\.(jpg|png|webp)}",
                web::get().to(image_host),
            )
            .route("/css/{filename:.*\\.css}", web::get().to(css_host))
            .route("/scripts/{filename:.*\\.js}", web::get().to(script_host))
            .route("/fonts/{filename:.*\\.ttf}", web::get().to(fonts_host))
            // favicon
            .route("/favicon.ico", web::get().to(favicon))
            // Index page routes
            .route("/", web::get().to(index_page))
            .route("/index.html", web::get().to(index_page))
    })
    .workers(config.server.workers)
    // TODO: figure out what this address should be so only the local subnet can access it. Not just local host
    .bind(("127.0.0.1", config.server.port))
    .expect("Could not bind server port")
    .run()
    .await?;

    Ok(())
}

async fn image_host(req: HttpRequest) -> io::Result<NamedFile> {
    let root_path: PathBuf = get_root_path("static/img/");
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let full_path = root_path.join(path);

    debug!("trying to serve image {}", full_path.display());
    NamedFile::open(full_path)
}

async fn css_host(req: HttpRequest) -> io::Result<NamedFile> {
    let root_path: PathBuf = get_root_path("static/css/");
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let full_path = root_path.join(path);

    debug!("trying to serve css {}", full_path.display());
    NamedFile::open(full_path)
}

async fn script_host(req: HttpRequest) -> io::Result<NamedFile> {
    let root_path: PathBuf = get_root_path("static/scripts/");
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let full_path = root_path.join(path);

    debug!("trying to serve script {}", full_path.display());
    NamedFile::open(full_path)
}

async fn fonts_host(req: HttpRequest) -> io::Result<NamedFile> {
    let root_path: PathBuf = get_root_path("static/fonts/");
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let full_path = root_path.join(path);

    debug!("trying to serve font {}", full_path.display());
    NamedFile::open(full_path)
}

async fn favicon(_req: HttpRequest) -> io::Result<NamedFile> {
    let favicon_path: PathBuf = get_root_path("static/img/logo-white.png");
    debug!("trying to serve favicon {}", favicon_path.display());
    NamedFile::open(favicon_path)
}

async fn index_page(
    app_data: web::Data<AppData>,
    _req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let context = Context::new();

    let page_text = app_data.templates.render("index.html", &context)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_text))
}

fn get_root_path(path: &str) -> PathBuf {
    let sub_path: PathBuf = path.parse().unwrap();
    env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join(sub_path)
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

#[post("/search")]
async fn post_search(info: web::Query<SearchParams>) -> Result<HttpResponse, Error> {
    info!("post search!!! {}", info.q);
    let results = get_search_results(&info.q).await?;
    Ok(HttpResponse::Ok().json(results))
}

#[get("/search")]
async fn get_search(
    app_data: web::Data<AppData>,
    info: web::Query<SearchParams>,
) -> Result<HttpResponse, Error> {
    info!("get search!!! {}", info.q);
    let results = get_search_results(&info.q).await?;

    // now to render the search results page
    let mut context = Context::new();
    context.insert("search_results", &results);
    context.insert("search_term", &info.q);

    let page_text = app_data.templates.render("search.html", &context)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_text))
}

async fn get_search_results(q: &str) -> Result<Vec<SearchResult>, Error> {
    let index = Index::load().await?;

    let results = index.search(q).await?;
    Ok(results)
}

fn load_templates() -> Result<Tera, Error> {
    let template_dir = env::current_exe()?
        .parent()
        .unwrap()
        .join("templates")
        .join("*.html");

    let mut tera = Tera::new(template_dir.to_str().unwrap())?;

    tera.autoescape_on(vec![]);

    let mut required_templates = HashSet::from(["index.html", "search.html", "header.html"]);

    info!("Loaded templates:");
    for template in tera.get_template_names() {
        info!("{}", template);
        required_templates.remove(template);
    }

    // fall back to load the search and index template from a string if we don't have it already.
    if !required_templates.is_empty() {
        warn!("Missing required templates: {:?}", required_templates);
        panic!("Missing required templates");
    }

    Ok(tera)
}
