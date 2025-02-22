use std::sync::LazyLock;

use crate::util::cli::DB_POOLS;
use actix_web::{HttpResponse, Responder};
use log::info;
use regex::Regex;
use rusqlite::named_params;
use serde_derive::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    data: Vec<Item>,
}

#[derive(Serialize)]
struct Item {
    dict: String,
    content: String,
}
static RE_CSS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(<link\s[^>]*?rel=['"]?stylesheet['"]?[^>]*?href=")([^"]*")"#).unwrap()
});
static RE_JS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(<script\s[^>]*?src=")([^"]*")"#).unwrap());

pub fn query(word: String) -> impl Responder {
    let w = word;
    let mut response = ApiResponse {
        data: Vec::with_capacity(DB_POOLS.len()),
    };
    for (db_id, pool) in DB_POOLS.iter() {
        let conn = pool.get().unwrap();
        let mut stmt = conn
            .prepare("select * from MDX_INDEX WHERE text= :word limit 1;")
            .unwrap();
        info!("query params={} from {}", &w, db_id);

        let mut rows = stmt.query(named_params! { ":word": w }).unwrap();
        let row = rows.next().unwrap();
        if let Some(row) = row {
            let html = row.get::<usize, String>(1).unwrap();
            let html = RE_CSS.replace_all(&html, r#"${1}files/$2"#).to_string();
            let html = RE_JS.replace_all(&html, r#"${1}files/$2"#).to_string();

            response.data.push(Item {
                dict: db_id.to_owned(),
                content: html,
            });
        }
    }
    HttpResponse::Ok().json(response)
}
