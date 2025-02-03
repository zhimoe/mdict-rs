use log::info;
use rusqlite::{named_params, Connection};

use crate::config::MDX_FILES;

pub fn query(word: String) -> String {
    let w = word;
    for file in MDX_FILES {
        let db_file = format!("{}{}", &file.to_string(), ".db");
        let conn = Connection::open(&db_file).unwrap();
        let mut stmt = conn
            .prepare("select * from MDX_INDEX WHERE text= :word limit 1;")
            .unwrap();
        info!("query params={}", &w);

        let mut rows = stmt.query(named_params! { ":word": w }).unwrap();
        let row = rows.next().unwrap();
        if let Some(row) = row {
            let def = row.get::<usize, String>(1).unwrap();
            return def;
        }
    }
    "not found".to_string()
}
