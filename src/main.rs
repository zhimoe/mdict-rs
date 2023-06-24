use std::io::{Read, Seek};
use std::path::PathBuf;

use actix_files;
use actix_web::{middleware, web, App, HttpServer};
use rusqlite::Connection;

use handlers::{handle_index, handle_lucky, handle_query};
use indexing::indexing;

use crate::mdict::mdx::Mdx;

mod checksum;
mod handlers;
mod indexing;
mod lucky;
mod mdict;
mod number;
mod query;
mod unpack;

const MDX_PATH: &str = "resources/mdx/en/牛津高阶8.mdx";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mdx = Mdx::new(MDX_PATH);
    let db_file = format!("{}{}", &mdx.filepath, ".db");

    if PathBuf::from(&db_file).exists() {
        std::fs::remove_file(&db_file).expect("remove old db file error");
        println!("old db file:{} removed", &db_file);
    }
    let mut conn = Connection::open(&db_file).unwrap();

    indexing(&mut conn, &mdx);
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
}

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .service(web::resource("/query").route(web::post().to(handle_query)))
            .service(web::resource("/lucky").route(web::get().to(handle_lucky)))
            .service(actix_files::Files::new("/", "resources/static").index_file("index.html")), // 必须放在最后，否则会屏蔽其他route
    );
}
