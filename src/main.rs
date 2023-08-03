use std::error::Error;
use std::path::PathBuf;

use actix_files;
use actix_web::{middleware, web, App, HttpServer};
use pretty_env_logger;
use rusqlite::Connection;

use config::static_path;
use handlers::{handle_lucky, handle_query};
use indexing::indexing_mdx_into_db;

use crate::config::mdx_path;
use crate::mdict::mdx::Mdx;

mod config;
mod handlers;
mod indexing;
mod lucky;
mod mdict;
mod query;
mod util;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let mdx = Mdx::new(mdx_path().unwrap().to_str().unwrap())?;
    let db_file = format!("{}{}", &mdx.filepath, ".db");

    if PathBuf::from(&db_file).exists() {
        std::fs::remove_file(&db_file).expect("remove old db file error");
        println!("old db file:{} removed", &db_file);
    }
    let mut conn = Connection::open(&db_file).unwrap();

    indexing_mdx_into_db(&mut conn, &mdx)?;
    println!("indexing record info done");

    println!("app serve on http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .service(web::resource("/query").route(web::post().to(handle_query)))
            .service(web::resource("/lucky").route(web::get().to(handle_lucky)))
            .service(
                actix_files::Files::new("/", static_path().unwrap().to_str().unwrap())
                    .index_file("index.html"),
            ), // 必须放在最后，否则会屏蔽其他route
    );
}
