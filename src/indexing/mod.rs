use anyhow::Context;
use rusqlite::{params, Connection};

use crate::mdict::mdx::Mdx;

pub fn indexing_mdx_into_db(conn: &mut Connection, mdx: &Mdx) -> anyhow::Result<()> {
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
    .with_context(|| "create table failed")?;

    let tx = conn
        .transaction()
        .with_context(|| "get transaction from connection failed")?;
    for r in &mdx.records {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?,?,?,?,?,?,?)",
            params![
                r.record_text,
                r.file_pos,
                r.compressed_size,
                r.decompressed_size,
                r.record_block_type,
                r.record_start,
                r.record_end,
                r.offset
            ],
        )
        .with_context(|| "insert MDX_INDEX table error")?;
    }
    tx.commit().with_context(|| "transaction commit error")?;
    Ok(())
}
