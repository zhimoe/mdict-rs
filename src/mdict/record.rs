/// item of Record Blocks, 即单词在mdx文件中的起始终止位置、字节长度等信息
#[derive(Debug)]
pub struct RecordIndex {
    pub key_text: String,
    pub file_pos: u32,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub record_block_type: u32,
    pub record_start: u32,
    pub record_end: u32,
    pub offset: u32,
}