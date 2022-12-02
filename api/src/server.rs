use core::sea_orm::{Database, DatabaseConnection};
use actix_files::Files as Fs;
use actix_web::{
    error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};

use crate::post::init as initPost;
use crate::member::init as initMember;
use listenfd::ListenFd;
use std::env;
use tera::Tera;

#[derive(Debug, Clone)]
pub struct AppState {
    pub templates: tera::Tera,
    pub conn: DatabaseConnection,
}

async fn not_found(data: web::Data<AppState>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // connection to database  
    let conn = Database::connect(&db_url).await.unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .default_service(web::route().to(not_found))
            .configure(initPost)
            .configure(initMember)
    }); 

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    };

    println!("Starting server at {}", server_url);
    server.run().await?;

    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {}", err);
    }
}
