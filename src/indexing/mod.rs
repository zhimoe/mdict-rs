use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use rusqlite::{Connection, params};

use crate::mdict::mdx::Mdx;
use tracing::info;

/// indexing all mdx files into db
pub(crate) fn indexing(files: &[&str], reindex: bool) {
    for file in files {
        let db_file = format!("{}{}", file.to_string(), ".db");
        if PathBuf::from(&db_file).exists() {
            if reindex {
                fs::remove_file(&db_file).expect("remove old db file error");
                info!("old db file:{} removed", &db_file);
                mdx_to_sqlite(file).unwrap();
            }
        } else {
            mdx_to_sqlite(file).unwrap();
        }
    }
}

/// mdx entries and definition to sqlite table
pub(crate) fn mdx_to_sqlite(file: &str) -> anyhow::Result<()> {
    let db_file = format!("{}{}", file, ".db");
    let mut conn = Connection::open(&db_file)?;
    let mdx = Mdx::new(&fs::read(file)?);

    conn.execute(
        "create table if not exists MDX_INDEX (
                text text primary key not null ,
                def text not null
         )",
        params![],
    )
    .with_context(|| "create table failed")?;
    info!("table crated for {:?}", &db_file);

    let tx = conn
        .transaction()
        .with_context(|| "get transaction from connection failed")?;

    for r in mdx.items() {
        tx.execute(
            "insert or replace into MDX_INDEX values (?,?)",
            params![r.text, r.definition],
        )
        .with_context(|| "insert MDX_INDEX table error")?;
    }
    tx.commit().with_context(|| "transaction commit error")?;
    conn.close().expect("close db connection failed");
    Ok(())
}
