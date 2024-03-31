use encoding::label::encoding_from_whatwg_label;
use nom::bytes::complete::take_till;
use nom::IResult;

use crate::mdict::header::parse_header;
use crate::mdict::keyblock::{parse_key_block_header, parse_key_block_info, parse_key_blocks, RecordEntry};
use crate::mdict::recordblock::{parse_record_blocks, record_block_parser, RecordBlockSize};

/// 一个record的定位信息：在buf中的offset和在block解压后的offset
//            block_offset_of_record
//                  │
//                  ▼
//              ┌───┬────────────┬───────┐
//    one block │   │   record   │       │
//              └───┼────────────┼───────┘
//              ▲   └────────────┘
//              │    record_csize
//              │
//     buf_offset_of_block
//
#[derive(Debug)]
struct RecordPosition {
    // buf offset of target block
    buf_offset_of_block: usize,
    // 计算方法：KeyEntry.buf_decompressed_offset - sum of all previous record blocks decompressed_size
    block_decompressed_offset: usize,
    // record compressed bytes size
    block_csize: usize,
    // record decompressed bytes size
    block_dsize: usize,
}

// todo: why can not be String?
#[derive(Debug)]
pub struct Record<'a> {
    text: &'a str,
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
    pub record_entries: Vec<RecordEntry>,
    pub record_blocks_size: Vec<RecordBlockSize>,
    pub record_block_buf: Vec<u8>,
    pub encoding: String,
    pub encrypted: String,
}


impl Mdx {
    /// let data = include_bytes!("/file.mdx");
    /// let mdx = Mdx::new(&data);
    pub fn new(data: &[u8]) -> Mdx {
        let (data, header) = parse_header(data).unwrap();

        let (data, kbh) = parse_key_block_header(data, &header).unwrap();
        let (data, key_blocks_size) = parse_key_block_info(data, kbh.key_block_info_len, &header).unwrap();
        let (data, record_entries) = parse_key_blocks(data, kbh.key_blocks_len, &header, &key_blocks_size).unwrap();
        let (data, record_blocks_size) = parse_record_blocks(data, &header).unwrap();
        Mdx {
            record_entries,
            record_blocks_size,
            record_block_buf: Vec::from(data),
            encoding: header.encoding,
            encrypted: header.encrypted,

        }
    }

    pub fn keys(&self) -> impl Iterator<Item=&RecordEntry> {
        return self.record_entries.iter();
    }


    pub fn items(&self) -> impl Iterator<Item=Record> {
        self.record_entries.iter().map(|entry| {
            let def = self.find_definition(entry);
            Record {
                text: &entry.text,
                definition: def,
            }
        })
    }

    /// 先看find_definition和record_offset方法理解文件结构
    fn find_definition(&self, entry: &RecordEntry) -> String {
        if let Some(position) = self.record_offset(entry) {
            let buf = &self.record_block_buf[position.buf_offset_of_block..];
            let (_, decompressed) = record_block_parser(position.block_csize, position.block_dsize)(buf).unwrap();
            //todo: why can not direct unwrap?
            let result: IResult<&[u8], &[u8]> =
                take_till(|x| x == 0)(&decompressed[position.block_decompressed_offset..]);
            let (_, record_decompressed) = result.unwrap();
            let decoder = encoding_from_whatwg_label(self.encoding.as_str()).unwrap();
            let text = decoder.decode(record_decompressed, encoding::DecoderTrap::Strict).unwrap();
            text
        } else {
            println!("find definition failed {:?}", entry.text);
            "".to_string()
        }
    }

    /// bytes structure: buf -> block -> record
    /// find record offset of input entry:
    /// 已知的是entry 在buf decompressed后的位置和每个block压缩和解压的长度
    /// 依次遍历blocks的size信息每个block的decompressed_size加起来如果小于entry.buf_decompressed_offset
    /// 说明目标entry所在的block还没遍历到
    fn record_offset(&self, record: &RecordEntry) -> Option<RecordPosition> {
        let mut pre_blocks_dsize_sum = 0;
        let mut pre_buf_csize_sum = 0;
        for block in &self.record_blocks_size {
            if record.buf_decompressed_offset <= pre_blocks_dsize_sum + block.dsize {
                // return Some((item_offset, block_offset, i));
                return Some(RecordPosition {
                    buf_offset_of_block: pre_buf_csize_sum,
                    block_decompressed_offset: record.buf_decompressed_offset - pre_blocks_dsize_sum,
                    block_csize: block.csize,
                    block_dsize: block.dsize,
                });
            } else {
                pre_blocks_dsize_sum += block.dsize;
                pre_buf_csize_sum += block.csize;
            }
        }
        None
    }
}


