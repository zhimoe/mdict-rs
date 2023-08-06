use anyhow::Context;
use rusqlite::{params, Connection};

use crate::mdict::mdx::Mdx;

pub fn indexing_mdx_into_db(conn: &mut Connection, mdx: &Mdx) -> anyhow::Result<()> {
    conn.execute(
        "create table if not exists MDX_INDEX (
                record_text text not null,
                block_file_pos integer,
                block_bytes_size integer,
                record_start integer,
                record_end integer,
                decompressed_offset integer
         )",
        params![],
    )
    .with_context(|| "create table failed")?;

    let tx = conn
        .transaction()
        .with_context(|| "get transaction from connection failed")?;
    for r in &mdx.records {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?,?,?,?,?)",
            params![
                r.record_text,
                r.block_file_pos,
                r.block_bytes_size,
                r.record_start,
                r.record_end,
                r.decompressed_offset
            ],
        )
        .with_context(|| "insert MDX_INDEX table error")?;
    }
    tx.commit().with_context(|| "transaction commit error")?;
    Ok(())
}
