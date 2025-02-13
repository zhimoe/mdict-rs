use crate::handlers::{handle_lucky, handle_query};
use crate::indexing::indexing;
use actix_files;
use actix_web::{middleware, web, App, HttpServer};
use pretty_env_logger;
use std::error::Error;
use util::cli::{get_static_path, ARGS, DB_FILES};

mod handlers;
mod indexing;
mod lucky;
mod mdict;
mod query;
mod util;

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .service(web::resource("/query").route(web::post().to(handle_query)))
            .service(web::resource("/lucky").route(web::get().to(handle_lucky)))
            // .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "max-age=86400")))
            .service(
                actix_files::Files::new("/", get_static_path().unwrap().to_str().unwrap())
                    .index_file("index.html"),
            ), // static file 必须放在最后，否则会屏蔽其他route
    );
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let _ = &*DB_FILES; // 访问 DB_FILES，确保 LazyLock 初始化

    indexing(false);
    if ARGS.generate_only {
        return Ok(());
    }
    let host = ARGS.host.unwrap_or_else(|| "127.0.0.1".parse().unwrap());
    let port = ARGS.port.unwrap_or(8181);
    println!("app serve on http://{host}:{port}");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind((host, port))?
    .run()
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}
