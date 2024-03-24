pub fn query(word: String) -> String {
    // let w = word;
    // let db_file = format!("{}{}", mdx_path().unwrap().to_str().unwrap(), ".db");
    // let conn = Connection::open(&db_file).unwrap();
    // let mut stmt = conn
    //     .prepare("select * from MDX_INDEX WHERE record_text= :word limit 1;")
    //     .unwrap();
    // info!("query params={}", &w);
    //
    // let mut rows = stmt.query(named_params! { ":word": w }).unwrap();
    // let row = rows.next().unwrap();
    // if let Some(row) = row {
    //     let idx = Record {
    //         record_text: row.get::<usize, String>(0).unwrap(),
    //         block_file_pos: row.get::<usize, u32>(1).unwrap(),
    //         block_bytes_size: row.get::<usize, u32>(2).unwrap(),
    //         record_start: row.get::<usize, i32>(3).unwrap() as u32,
    //         record_end: row.get::<usize, i32>(4).unwrap() as u32,
    //         decompressed_offset: row.get::<usize, i32>(5).unwrap() as u32,
    //     };
    //
    //     let mut reader =
    //         BufReader::new(File::open(&mdx_path().unwrap().to_str().unwrap()).unwrap());
    //     reader
    //         .seek(SeekFrom::Start(idx.block_file_pos as u64))
    //         .expect("reader seek error");
    //
    //     let mut record_block_compressed: Vec<u8> = vec![0; idx.block_bytes_size as usize];
    //     reader
    //         .read_exact(&mut record_block_compressed)
    //         .expect("read record_block_compressed error ");
    //     return Mdx::extract_definition(&mut record_block_compressed, &idx);
    // }
    return "not found".to_string();
}
