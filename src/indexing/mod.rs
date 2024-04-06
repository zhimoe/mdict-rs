use anyhow::Context;
use rusqlite::{params, Connection};

use crate::mdict::mdx::Mdx;

/// mdx entries and definition to sqlite table
pub fn mdx_to_sqlite(conn: &mut Connection, mdx: &Mdx) -> anyhow::Result<()> {
    conn.execute(
        "create table if not exists MDX_INDEX (
                text text not null,
                def text not null
         )",
        params![],
    )
    .with_context(|| "create table failed")?;
    println!("{}", "table created");

    let tx = conn
        .transaction()
        .with_context(|| "get transaction from connection failed")?;

    for r in mdx.items() {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?)",
            params![r.text, r.definition],
        )
        .with_context(|| "insert MDX_INDEX table error")?;
    }
    println!("{}", "start commit");
    tx.commit().with_context(|| "transaction commit error")?;
    Ok(())
}
