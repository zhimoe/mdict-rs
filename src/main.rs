use std::arch::asm;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use actix_files as fs;
use actix_web::web::service;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use rusqlite::{named_params, params, Connection};
use serde_derive::Deserialize;

use crate::mdict::mdx::Mdx;
use crate::mdict::record::RecordIndex;

mod checksum;
mod lucky;
mod mdict;
mod number;
mod unpack;

const MDX_PATH: &str = "resources/mdx/en/牛津高阶8.mdx";

fn query(word: String) -> String {
    let w = word;
    let db_file = format!("{}{}", MDX_PATH, ".db");
    let conn = Connection::open(&db_file).unwrap();
    let mut stmt = conn
        .prepare("select * from MDX_INDEX WHERE key_text= :word;")
        .unwrap();
    println!("query params={}", &w);
    let mut rows = stmt.query(named_params! { ":word": w }).unwrap();
    let row = rows.next().unwrap();
    if let Some(row) = row {
        let idx = RecordIndex {
            key_text: row.get::<usize, String>(0).unwrap(),
            file_pos: row.get::<usize, u32>(1).unwrap() as u32,
            compressed_size: row.get::<usize, u32>(2).unwrap() as u32,
            decompressed_size: row.get::<usize, u32>(3).unwrap() as u32,
            record_block_type: row.get::<usize, u8>(4).unwrap() as u32,
            record_start: row.get::<usize, i32>(5).unwrap() as u32,
            record_end: row.get::<usize, i32>(6).unwrap() as u32,
            offset: row.get::<usize, i32>(7).unwrap() as u32,
        };

        let mut reader = BufReader::new(File::open(&MDX_PATH).unwrap());
        reader
            .seek(SeekFrom::Start(idx.file_pos as u64))
            .expect("reader seek error");

        let mut record_block_compressed: Vec<u8> = vec![0; idx.compressed_size as usize];
        reader
            .read_exact(&mut record_block_compressed)
            .expect("read record_block_compressed error ");
        return Mdx::extract_definition(
            &mut record_block_compressed,
            idx.record_start as usize,
            idx.record_end as usize,
            idx.offset as usize,
        );
    }
    return "not found".to_string();
}

fn indexing(conn: &mut Connection, mdx: &Mdx) {
    conn.execute(
        "create table if not exists MDX_INDEX (
                key_text text not null,
                file_pos integer,
                compressed_size integer,
                decompressed_size integer,
                record_block_type integer,
                record_start integer,
                record_end integer,
                offset integer
         )",
        params![],
    )
    .expect("create db error");

    let tx = conn.transaction().unwrap();
    for r in &mdx.records {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?,?,?,?,?,?,?)",
            params![
                r.key_text,
                r.file_pos,
                r.compressed_size,
                r.decompressed_size,
                r.record_block_type,
                r.record_start,
                r.record_end,
                r.offset
            ],
        )
        .expect("insert MDX_INDEX table error");
    }
    tx.commit().expect("transaction commit error");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mdx = Mdx::new(MDX_PATH);
    let db_file = format!("{}{}", &mdx.filepath, ".db");

    if PathBuf::from(&db_file).exists() {
        std::fs::remove_file(&db_file).expect("remove old db file error");
        println!("old db file:{} removed", &db_file);
    }
    let mut conn = Connection::open(&db_file).unwrap();
    println!("start indexing mdx to db");
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
            .service(web::resource("/").route(web::get().to(index)))
            .service(fs::Files::new("/", "resources/static")), // 必须放在最后，否则会屏蔽其他route
    );
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../resources/static/index.html")))
}

#[derive(Deserialize, Debug)]
struct QueryForm {
    word: String,
}

async fn handle_query(params: web::Form<QueryForm>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("{}", query(params.word.clone()))))
}

async fn handle_lucky() -> Result<HttpResponse> {
    let word = lucky::lucky_word();
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("{}", query(word))))
}
