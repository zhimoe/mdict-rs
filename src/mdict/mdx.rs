use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use TryInto;

use anyhow::Context;
use log::info;

use crate::mdict::key::KeyBlockCodecInfo;
use crate::util::checksum::adler32_checksum;
use crate::util::number::{read_number_from_be_bytes, NumberFromBeBytes};
use crate::util::zlib::decompress;

use super::header::Header;
use super::key::RecordIndex;
use super::record::Record;

/// Mdx file结构包含三部分
/// header 元信息
/// keys 词条record在block中的索引信息
/// records 词条释义信息
/// 详细结构见 https://bitbucket.org/xwang/mdict-analysis/src/master/MDX.svg
pub struct Mdx {
    pub filepath: String,
    pub header: Header,
    // mdx file passcode
    pub passcode: String,
    pub records: Vec<Record>,
}

impl Mdx {
    pub fn new(file: &str) -> anyhow::Result<Mdx> {
        let mut reader = BufReader::new(File::open(&file)?);

        let mut header_len_bytes = [0; 4];
        reader
            .read_exact(&mut header_len_bytes)
            .with_context(|| "read header len bytes failed")?;
        let header_len = u32::from_be_bytes(header_len_bytes.try_into().unwrap());
        info!("the header len is {}", &header_len);

        let mut header_bytes = vec![0; header_len as usize];
        reader
            .read_exact(&mut header_bytes)
            .with_context(|| "read header bytes of dict info error")?;

        // reade 4 bytes: adler32 checksum of header, in little endian
        let mut adler32_bytes = [0; 4];
        reader
            .read_exact(&mut adler32_bytes)
            .with_context(|| "read adler32_bytes error")?;

        if adler32_checksum(
            &header_bytes,
            u32::from_le_bytes(adler32_bytes.try_into().unwrap()),
        ) {
            info!("header bytes adler32 checksum success")
        } else {
            panic!("unrecognized mdx file format");
        }

        let mut header = Header::new(header_bytes)?;
        let pos = reader
            .seek(SeekFrom::Current(0))
            .with_context(|| "get current file position error")?;
        info!("key_block_offset is {}", &pos);
        header.key_block_offset = pos;

        // parse the key block part
        let number_width: u8 = header.number_width();
        let mut key_block_meta_bytes = vec![0; header.key_block_meta_bytes_size() as usize];
        reader
            .read_exact(&mut key_block_meta_bytes)
            .with_context(|| "read key block info meta bytes error")?;
        // adler32 checksum of previous 5 bytes, in big endian, only version >= 2.0
        if header.engine_version >= 2.0 {
            let mut adler32_bytes = [0; 4];
            reader
                .read_exact(&mut adler32_bytes)
                .with_context(|| "read exact error")?;

            if adler32_checksum(
                &key_block_meta_bytes,
                u32::from_be_bytes(adler32_bytes.try_into().unwrap()),
            ) {
                info!("key block info adler32 checksum success")
            } else {
                panic!("key block info adler32 checksum error, unrecognized format");
            }
        }

        let mut meta_numbers = NumberFromBeBytes::new(&key_block_meta_bytes, number_width);

        let __key_blocks_count = meta_numbers.next();
        let __entries_count = meta_numbers.next();
        if header.engine_version >= 2.0 {
            let __key_block_info_decompressed_bytes_len = meta_numbers.next();
        }

        let key_block_info_bytes_len = meta_numbers.next().unwrap();
        let key_blocks_bytes_num = meta_numbers.next().unwrap();

        let mut key_block_info_bytes = vec![0; key_block_info_bytes_len as usize];
        reader
            .read_exact(&mut key_block_info_bytes)
            .with_context(|| "read exact error")?;

        let mut key_block_list_bytes = vec![0; key_blocks_bytes_num as usize];
        reader
            .read_exact(&mut key_block_list_bytes)
            .with_context(|| "read exact error")?;

        header.record_block_offset = reader
            .seek(SeekFrom::Current(0))
            .with_context(|| "get current file position error")?;

        let key_block_codec_size_list =
            KeyBlockCodecInfo::list_from_bytes(&key_block_info_bytes, &header);
        info!(
            "key_block_codec_size_list len= {}",
            &key_block_codec_size_list.len(),
        );

        let key_id_text_list = RecordIndex::list_from_bytes_and_codec_info(
            &key_block_list_bytes,
            &key_block_codec_size_list,
        );

        // parse record block
        let record_blocks_num = read_number_from_be_bytes(&mut reader, number_width);
        let __entries_num = read_number_from_be_bytes(&mut reader, number_width);
        let record_block_codec_list_bytes_size =
            read_number_from_be_bytes(&mut reader, number_width);
        let _record_block_list_bytes_size = read_number_from_be_bytes(&mut reader, number_width);

        let mut record_block_codec_size_list: Vec<(usize, usize)> = vec![];
        let mut codec_bytes_counter: usize = 0;
        // read all record_block_info bytes
        for _i in 0..record_blocks_num {
            let co_size = read_number_from_be_bytes(&mut reader, number_width);
            let dec_size = read_number_from_be_bytes(&mut reader, number_width);
            record_block_codec_size_list.push((co_size, dec_size));
            codec_bytes_counter += number_width as usize * 2;
        }
        assert_eq!(codec_bytes_counter, record_block_codec_list_bytes_size);

        // start read record block, decompress it
        let mut record_list: Vec<Record> = vec![]; // important!
        let mut i: usize = 0;
        let mut offset: usize = 0;

        for (c_size, d_size) in record_block_codec_size_list {
            let cur_pos = reader
                .seek(SeekFrom::Current(0))
                .with_context(|| "get current file position error")?;

            let mut record_block_bytes_compressed: Vec<u8> = vec![0; c_size];
            reader
                .read_exact(&mut record_block_bytes_compressed)
                .with_context(|| "read exact error")?;

            // split record block into record according to the offset info from key block
            while i < key_id_text_list.len() {
                let key_index = &key_id_text_list[i];
                let start = key_index.start as usize;
                if start - offset >= d_size {
                    break;
                }
                let record_end: usize;
                if i < key_id_text_list.len() - 1 {
                    record_end = key_id_text_list[i + 1].start as usize;
                } else {
                    record_end = d_size + offset;
                }
                let idx = Record {
                    record_text: key_index.text.to_string(),
                    block_file_pos: cur_pos as u32,
                    block_bytes_size: c_size as u32,
                    record_start: key_index.start.clone() as u32,
                    record_end: record_end as u32,
                    decompressed_offset: offset as u32,
                };
                i += 1;

                // let mut record = &record_block_decompressed[(start - offset)..(record_end - offset)];
                // let content = String::from_utf8_lossy(record);
                record_list.push(idx)
            }
            offset += d_size;
        }

        Ok(Mdx {
            filepath: file.to_string(),
            header,
            passcode: "".to_string(),
            records: record_list,
        })
    }

    /// util function, extract word definitions from bytes
    pub fn extract_definition(block_bytes: &Vec<u8>, record: &Record) -> String {
        let record_block_type = &block_bytes[0..4];
        assert_eq!(b"\x02\x00\x00\x00", record_block_type);

        let mut block_decompressed = Vec::new();
        decompress(&block_bytes[8..], &mut block_decompressed);

        let adler32_bytes = &block_bytes[4..8];
        if !adler32_checksum(
            &block_decompressed,
            u32::from_be_bytes(adler32_bytes.try_into().unwrap()),
        ) {
            panic!("record block adler32 checksum failed");
        }

        let s: usize = (record.record_start - record.decompressed_offset) as usize;
        let e: usize = (record.record_end - record.decompressed_offset) as usize;
        let record = &block_decompressed[s..e];

        return String::from_utf8_lossy(record).to_string();
    }
}

/*
fn decompress_record_block_bytes(record_block_bytes_compressed: &mut Vec<u8>) -> (Vec<u8>, i32) {
    let record_block_type = &record_block_bytes_compressed[0..4];
    let adler32_bytes = &record_block_bytes_compressed[4..8];
    let mut record_block_decompressed = Vec::new();
    let mut _type = 2;
    match record_block_type {
        b"\x02\x00\x00\x00" => {
            _type = 2;
            decompress(
                &record_block_bytes_compressed[8..],
                &mut record_block_decompressed,
            );

            if !adler32_checksum(
                &record_block_decompressed,
                u32::from_be_bytes(adler32_bytes.try_into().unwrap()),
            ) {
                panic!("record block adler32 checksum failed");
            }
        }
        b"\x01\x00\x00\x00" => {
            println!("\x01\x00\x00\x00")
        }
        b"\x00\x00\x00\x00" => {
            println!("\x00\x00\x00\x00")
        }
        _ => {}
    }
    (record_block_decompressed, _type)
}
 */
