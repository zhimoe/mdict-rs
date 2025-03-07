use crate::mdict::mdx::Mdx;
use crate::util::cli::get_dicts_mdx;
use anyhow::Context;
use log::info;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

/// indexing all mdx files into db
pub(crate) fn indexing(reindex: bool) {
    let files = if let Some(v) = get_dicts_mdx().unwrap() {
        v
    } else {
        return;
    };
    for file in files {
        let db_file = file.with_extension("db");
        if db_file.exists() {
            if reindex {
                fs::remove_file(&db_file).expect("remove old db file error");
                info!("old db file:{} removed", &db_file.to_string_lossy());
                mdx_to_sqlite(file).unwrap();
            }
        } else {
            mdx_to_sqlite(file).unwrap();
        }
    }
}

/// mdx entries and definition to sqlite table
pub(crate) fn mdx_to_sqlite(file: PathBuf) -> anyhow::Result<()> {
    let db_file = file.with_extension("db");
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
    println!("table crated for {:?}", &db_file);

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
