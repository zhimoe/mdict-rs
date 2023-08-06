/// 单词在mdx文件中的起始终止位置、字节长度等信息
#[derive(Debug)]
pub struct Record {
    // 词条
    pub record_text: String,
    // record所在block在文件的起始位置
    pub block_file_pos: u32,
    // block 字节数
    pub block_bytes_size: u32,
    // record在block的压缩字节起始位置
    pub record_start: u32,
    // record在block的压缩字节结束位置
    pub record_end: u32,
    // record_start record_end在解压缩的偏移量
    pub decompressed_offset: u32,
}
