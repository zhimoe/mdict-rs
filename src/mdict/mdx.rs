use crate::mdict::header::parse_header;
use crate::mdict::keyblock::{
    parse_key_block_header, parse_key_block_info, parse_key_blocks, RecordDeBufOffset,
};
use crate::mdict::recordblock::{parse_record_blocks, record_block_parser, RecordBlockSize};
use nom::Parser;

/// 一个record的定位信息：在buf(buf表示所有record_block的bytes)中的offset和在block解压后的offset
/// draw with: https://asciiflow.com/#/
//                   ◄──block_csize───►
//                   ┌────────────────┐
//            block  │                │
//                   └────────────────┘
//                   ▲
//           block_start_in_buf
//
//                   ◄──── block_dsize ───────►
//                   ┌───┬────────────┬───────┐
//     block_decomp  │   │   record   │       │
//                   └───┴────────────┴───────┘
//                       ▲
//           record_start_in_de_block
//
#[derive(Debug)]
pub struct RecordOffsetInfo {
    pub(crate) text: String,
    // record所在block在buf的offset 截取block使用
    block_offset_in_buf: usize,
    // 解析block使用
    block_csize: usize,
    block_dsize: usize,
    // record在解压后的block的offset 和 end
    record_start_in_de_block: usize,
    record_end_in_de_block: usize,
}

// todo: why can not be String?
#[derive(Debug)]
pub struct Record<'a> {
    pub(crate) text: &'a str,
    pub(crate) definition: String,
}

/// MDX 详细结构见 https://bitbucket.org/xwang/mdict-analysis/src/master/MDX.svg
/// MDX file 结构
/// header: 得到 version encoding encrypted
/// key block header: entry number and checksum
/// key block size info: every key block compressed and decompressed size, for parse key block bytes
/// key block bytes: 根据上面的key block info得到的（csize,dsize）解析得到 Entry list
/// record header: record block size, entry number, record block info size, record block size
/// record block size info: every record block compressed and decompressed size, 用于解析下面的record block
/// record block bytes: entry and definition bytes, parsed by RecordEntry and RecordBlockSize
/// record: 是一条释义
pub struct Mdx {
    pub records_offset: Vec<RecordOffsetInfo>,
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
        let (data, key_blocks_size) =
            parse_key_block_info(data, kbh.key_block_info_len, &header).unwrap();
        let (data, entries) =
            parse_key_blocks(data, kbh.key_blocks_len, &header, &key_blocks_size).unwrap();
        let (data, record_blocks_size) = parse_record_blocks(data, &header).unwrap();

        //计算position耗时，一次计算就保存下来
        let offset: Vec<RecordOffsetInfo> = records_offset(&entries, &record_blocks_size);

        Mdx {
            records_offset: offset,
            record_block_buf: Vec::from(data),
            encoding: header.encoding,
            encrypted: header.encrypted,
        }
    }

    #[allow(unused)]
    pub fn entries(&self) -> impl Iterator<Item = &RecordOffsetInfo> {
        return self.records_offset.iter();
    }

    pub fn items(&self) -> impl Iterator<Item = Record> {
        self.records_offset.iter().map(|rs| {
            let def = self.find_definition(&rs);
            Record {
                text: &rs.text,
                definition: def,
            }
        })
    }

    fn find_definition(&self, rs: &RecordOffsetInfo) -> String {
        // block bytes with tail
        let block_buf = &self.record_block_buf[rs.block_offset_in_buf..];

        let (_, block_decompressed) = record_block_parser(rs.block_csize, rs.block_dsize)
            .parse(block_buf)
            .unwrap();

        let record_decompressed =
            &block_decompressed[rs.record_start_in_de_block..rs.record_end_in_de_block];

        let def = String::from_utf8_lossy(record_decompressed).to_string();

        return def;
    }
}

/// bytes structure: buf -> block -> record(entry)
fn records_offset(
    records_debuf_index: &Vec<RecordDeBufOffset>,
    record_blocks_size: &Vec<RecordBlockSize>,
) -> Vec<RecordOffsetInfo> {
    let mut positions: Vec<RecordOffsetInfo> = vec![];
    let mut i: usize = 0;
    let mut pre_blocks_dsize_sum = 0;
    let mut pre_blocks_csize_sum = 0;
    // 同时开始遍历record_blocks_size和entries，每个block包含0或n个entry，
    // 当entry的buf_decompressed_offset > pre_blocks_dsize_sum时 说明当前block已经遍历结束
    for block in record_blocks_size {
        while i < records_debuf_index.len() {
            let record = &records_debuf_index[i];

            // 当前entry已经属于下一个block，注意等于号
            if record.record_offset_in_debuf >= pre_blocks_dsize_sum + block.dsize {
                break;
            }

            let record_end_in_de_block;
            if i < records_debuf_index.len() - 1 {
                let next_entry = &records_debuf_index[i + 1];
                record_end_in_de_block = next_entry.record_offset_in_debuf - pre_blocks_dsize_sum;
            } else {
                // last entry
                record_end_in_de_block = block.dsize
            }

            positions.push(RecordOffsetInfo {
                text: record.text.to_string(),
                block_offset_in_buf: pre_blocks_csize_sum,
                block_csize: block.csize,
                block_dsize: block.dsize,
                record_start_in_de_block: record.record_offset_in_debuf - pre_blocks_dsize_sum,
                record_end_in_de_block: record_end_in_de_block,
            });
            i += 1;
        }
        pre_blocks_dsize_sum += block.dsize;
        pre_blocks_csize_sum += block.csize;
    }
    return positions;
}
