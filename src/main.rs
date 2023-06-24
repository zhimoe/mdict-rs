use std::arch::asm;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use actix_files as fs;
use actix_web::web::service;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use rusqlite::{named_params, params, Connection};
use serde_derive::Deserialize;

use handlers::{handle_index, handle_lucky, handle_query};
use indexing::indexing;

use crate::mdict::mdx::Mdx;
use crate::mdict::record::RecordIndex;

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
            .service(web::resource("/").route(web::get().to(handle_index)))
            .service(fs::Files::new("/", "resources/static")), // 必须放在最后，否则会屏蔽其他route
    );
}
