use std::error::Error;

use actix_files;
use actix_web::{middleware, web, App, HttpServer};
use pretty_env_logger;

use crate::config::{static_path, MDX_FILES};
use crate::handlers::{handle_lucky, handle_query};
use crate::indexing::indexing;

mod config;
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
                actix_files::Files::new("/", static_path().unwrap().to_str().unwrap())
                    .index_file("index.html"),
            ), // static file 必须放在最后，否则会屏蔽其他route
    );
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    indexing(MDX_FILES, false);

    println!("app serve on http://127.0.0.1:8181");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind(("127.0.0.1", 8181))?
    .run()
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}
