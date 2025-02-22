use crate::util::cli::DB_POOLS;
use actix_web::{HttpResponse, Responder};
use log::info;
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
            let def = row.get::<usize, String>(1).unwrap();
            response.data.push(Item {
                dict: db_id.to_owned(),
                content: def,
            });
        }
    }
    HttpResponse::Ok().json(response)
}
