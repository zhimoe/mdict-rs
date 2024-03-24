use crate::mdict::header::parse_header;
use crate::mdict::keyblock::{KeyEntry, parse_key_block_header, parse_key_block_info, parse_key_blocks};
use crate::mdict::recordblock::{parse_record_blocks, RecordBlockSize};

/// 一个record的定位信息：在buf中的offset和在block解压后的offset
#[derive(Debug)]
struct RecordOffset {
    // sum of all previous record blocks size
    buf_offset: usize,
    // 计算方法：KeyEntry.buf_decompressed_offset - sum of all previous record blocks decompressed_size
    block_decompressed_offset: usize,
    // record bytes len
    compressed_len: usize,
    // record decompressed len
    decompressed_len: usize,
}

#[derive(Debug)]
pub struct Record {
    key: String,
    definition: String,
}

/// MDX 详细结构见 https://bitbucket.org/xwang/mdict-analysis/src/master/MDX.svg
/// MDX file 结构
/// header: version encoding encrypted
/// key block header: entry number and checksum
/// key block info: every key block compressed and decompressed size, for parse key block bytes
/// key block list: key_text,  key_offset: record offset in on record block? KeyEntry KeyBlock
/// record header: record block size, entry number, record block info size, record block size
/// record block size list: every record block compressed and decompressed size
/// record block bytes:   for (c_size, d_size) in record_block_cd_size_list
#[derive(Debug)]
pub struct Mdx {
    pub key_entries: Vec<KeyEntry>,
    pub record_blocks_size: Vec<RecordBlockSize>,
    pub records_buf: Vec<u8>,
    pub encoding: String,
    pub encrypted: String,
}


impl Mdx {
    /// let data = include_bytes!("/file.mdx");
    /// let mdx = Mdx::new(&data);
    pub fn new(data: &[u8]) -> Mdx {
        // parse header
        let (data, header) = parse_header(data).unwrap();
        // parse key block
        let (data, key_block_header) = parse_key_block_header(data, &header).unwrap();
        let (data, key_blocks_size) = parse_key_block_info(data, key_block_header.key_block_info_len, &header).unwrap();
        let (data, key_entries) = parse_key_blocks(
            data,
            key_block_header.key_blocks_len,
            &header,
            &key_blocks_size,
        ).unwrap();
        let (data, record_blocks_size) = parse_record_blocks(data, &header).unwrap();
        Mdx {
            key_entries,
            record_blocks_size,
            records_buf: Vec::from(data),
            encoding: header.encoding,
            encrypted: header.encrypted,

        }
    }

    // pub fn items(&self) -> impl Iterator<Item=Record> {
    //     self.key_entries.iter().map(|entry| {
    //         let def = self.find_definition(entry);
    //         Record {
    //             key: &entry.text,
    //             definition: def,
    //         }
    //     })
    // }

    pub fn keys(&self) -> impl Iterator<Item=&KeyEntry> {
        return self.key_entries.iter();
    }
}


