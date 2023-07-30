use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use log::info;
use rusqlite::{named_params, Connection};

use crate::config::mdx_path;
use crate::mdict::mdx::Mdx;
use crate::mdict::record::RecordIndex;

pub fn query(word: String) -> String {
    let w = word;
    let db_file = format!("{}{}", mdx_path().unwrap().to_str().unwrap(), ".db");
    let conn = Connection::open(&db_file).unwrap();
    let mut stmt = conn
        .prepare("select * from MDX_INDEX WHERE key_text= :word;")
        .unwrap();
    info!("query params={}", &w);
    let mut rows = stmt.query(named_params! { ":word": w }).unwrap();
    let row = rows.next().unwrap();
    if let Some(row) = row {
        let idx = RecordIndex {
            key_text: row.get::<usize, String>(0).unwrap(),
            file_pos: row.get::<usize, u32>(1).unwrap() as u32,
            compressed_size: row.get::<usize, u32>(2).unwrap() as u32,
            decompressed_size: row.get::<usize, u32>(3).unwrap() as u32,
            record_block_type: row.get::<usize, u8>(4).unwrap() as u32,
            record_start: row.get::<usize, i32>(5).unwrap() as u32,
            record_end: row.get::<usize, i32>(6).unwrap() as u32,
            offset: row.get::<usize, i32>(7).unwrap() as u32,
        };

        let mut reader =
            BufReader::new(File::open(&mdx_path().unwrap().to_str().unwrap()).unwrap());
        reader
            .seek(SeekFrom::Start(idx.file_pos as u64))
            .expect("reader seek error");

        let mut record_block_compressed: Vec<u8> = vec![0; idx.compressed_size as usize];
        reader
            .read_exact(&mut record_block_compressed)
            .expect("read record_block_compressed error ");
        return Mdx::extract_definition(
            &mut record_block_compressed,
            idx.record_start as usize,
            idx.record_end as usize,
            idx.offset as usize,
        );
    }
    return "not found".to_string();
}
