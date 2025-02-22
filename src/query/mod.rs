use std::sync::LazyLock;

use crate::util::cli::DB_POOLS;
use actix_web::{HttpResponse, Responder};
use log::info;
use regex::{Captures, Regex};
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
static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?xi)
        (                       # 分组1：整个属性部分
            (href|src)          # 属性名
            \s*=\s*"            # 等号和引号
        )
        (                       # 分组3：需要处理的路径
            (?:[^"f]|f[^i]|fi[^l]|fil[^e]|file[^s])*?  # 排除以files/开头的路径
            (.*?\.(?:css|js))   # 目标文件扩展名
        )
        "#).unwrap()
});

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
            let html = RE.replace_all(&html, |caps: &Captures| {
                // 仅当路径不以files/开头时才添加前缀
                let full_path = &caps[3];
                if !full_path.starts_with("files/") {
                    format!("{}{}{}", &caps[1], "files/", full_path)
                } else {
                    caps[0].to_string()  // 保留原内容
                }
            });

            response.data.push(Item {
                dict: db_id.to_owned(),
                content: html.into_owned(),
            });
        }
    }
    HttpResponse::Ok().json(response)
}
