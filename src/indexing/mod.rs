use rusqlite::{params, Connection};

use crate::mdict::mdx::Mdx;

pub(crate) fn indexing(conn: &mut Connection, mdx: &Mdx) {
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
