use log::debug;
use ripemd128::{Digest, Ripemd128};

use crate::mdict::header::Header;
use crate::util::checksum::adler32_checksum;
use crate::util::zlib::decompress;

/// record index, 即mdx中所有词条索引
#[derive(Debug)]
pub struct RecordIndex {
    // start position of record
    pub index: u64,
    // text of record
    pub text: String,
}

/// 每个KeyBlock bytes压缩前后的大小，一个KeyBlock包含多个RecordIndex
#[derive(Debug)]
pub struct KeyBlockCodecInfo {
    pub compressed_size: usize,
    pub decompressed_size: usize,
}

impl KeyBlockCodecInfo {
    /// 将key block info bytes 解析成 KeyBlockItemCodecSize list
    pub fn list_from_bytes(
        key_block_info_bytes: &Vec<u8>,
        header: &Header,
    ) -> Vec<KeyBlockCodecInfo> {
        let v2_prefix_bytes = &key_block_info_bytes[0..4];
        let mut adler32_bytes = &key_block_info_bytes[4..8];
        let data = &key_block_info_bytes[8..];
        let mut decrypted_compressed_bytes = vec![0; data.len()];
        let mut decompressed_key_block_info_bytes = Vec::new();
        if header.engine_version >= 2.0 {
            assert_eq!(b"\x02\x00\x00\x00", v2_prefix_bytes);
            let encrypted = header.encrypted.parse::<u32>().unwrap();
            // 如果key block info加密了，需要先解密
            if &encrypted & 0x02 == 0x02 {
                debug!("key block info bytes encrypted, decrypt first");
                let key = KeyBlockCodecInfo::get_decrypt_key(&mut adler32_bytes);
                let mut previous: u8 = 0x36;
                for i in 0..data.len() {
                    let mut t = (data[i] >> 4 | data[i] << 4) & 0xff;
                    t = t ^ previous ^ (i & 0xff) as u8 ^ key[i % key.len()];
                    previous = data[i].clone();
                    decrypted_compressed_bytes[i] = t;
                }
            } else {
                debug!("key block info bytes not encrypted");
                decrypted_compressed_bytes = Vec::from(data.clone());
            }

            // data now is decrypted, then decompress
            decompress(
                &decrypted_compressed_bytes,
                &mut decompressed_key_block_info_bytes,
            );

            if !adler32_checksum(
                &decompressed_key_block_info_bytes,
                u32::from_be_bytes(adler32_bytes.try_into().unwrap()),
            ) {
                panic!("key_block_info adler32 checksum failed!")
            }
        }

        //start decode
        let mut _num_entries = 0u64;
        let mut byte_width = 1;
        let mut text_term = 0;
        if header.engine_version >= 2.0 {
            byte_width = 2;
            text_term = 1;
        }
        let num_width = 8;
        let mut i = 0;
        let mut key_block_info_list: Vec<KeyBlockCodecInfo> = vec![];
        while i < decompressed_key_block_info_bytes.len() {
            _num_entries += u64::from_be_bytes(
                (&decompressed_key_block_info_bytes[i..(i + num_width)])
                    .try_into()
                    .unwrap(),
            );

            i += num_width;
            let text_head_size = u16::from_be_bytes(
                (&decompressed_key_block_info_bytes[i..(i + byte_width)])
                    .try_into()
                    .unwrap(),
            );
            i += byte_width;
            i += (text_head_size + text_term) as usize;
            let text_tail_size = u16::from_be_bytes(
                (&decompressed_key_block_info_bytes[i..(i + byte_width)])
                    .try_into()
                    .unwrap(),
            );
            i += byte_width;
            i += (text_tail_size + text_term) as usize;

            let key_block_compressed_size = u64::from_be_bytes(
                (&decompressed_key_block_info_bytes[i..(i + num_width)])
                    .try_into()
                    .unwrap(),
            );
            i += num_width;
            let key_block_decompressed_size = u64::from_be_bytes(
                (&decompressed_key_block_info_bytes[i..(i + num_width)])
                    .try_into()
                    .unwrap(),
            );
            i += num_width;
            key_block_info_list.push(KeyBlockCodecInfo {
                compressed_size: key_block_compressed_size as usize,
                decompressed_size: key_block_decompressed_size as usize,
            })
        }
        return key_block_info_list;
    }

    fn get_decrypt_key(adler32_bytes: &mut &[u8]) -> Vec<u8> {
        let fix: Vec<u8> = vec![0x95, 0x36, 0x00, 0x00]; //0x3695 in little endian
                                                         // create a RIPEMD-128 hasher instance
        let mut hasher = Ripemd128::new();
        hasher.input([&adler32_bytes, &fix[..]].concat());
        // acquire hash digest in the form of GenericArray,
        // which in this case is equivalent to [u8; 16]
        let ga = hasher.result();
        ga.as_slice().iter().cloned().collect()
    }
}

/// key block 信息，包含三段
/// meta： 元信息 5x8 或者 4x4 字节
/// key block info: 每个key block item压缩后和解压后的bytes size
/// key block item: 即KeyIndex部分
#[derive(Debug)]
pub struct KeyBlock {
    pub key_block_item_count: u64,
    pub entries_count: u64,
    // only when version >= 2.0
    pub key_block_info_bytes_decompressed_size: u64,
    pub key_block_info_bytes_size: u64,
    pub key_block_item_codec_size_list: Vec<KeyBlockCodecInfo>,
    pub key_block_item_list: Vec<RecordIndex>,
}

impl RecordIndex {
    /// 将整个key blocks bytes 解析成一个 Vec<RecordIndex>
    pub fn list_from_bytes_and_codec_info(
        key_blocks_bytes: &Vec<u8>,
        key_blocks_codec_info: &Vec<KeyBlockCodecInfo>,
    ) -> Vec<RecordIndex> {
        let mut whole_index_list: Vec<RecordIndex> = vec![];
        let mut i: usize = 0;
        let mut end: usize = i;
        for one_codec_info in key_blocks_codec_info {
            let start = i;
            end += one_codec_info.compressed_size;

            let compressed_key_block_bytes = &key_blocks_bytes[start..end];

            let key_block_type = &compressed_key_block_bytes[0..4];
            // let _adler32_bytes = &compressed_key_block_bytes[4..8];
            let mut key_block = Vec::new();
            match key_block_type {
                b"\x02\x00\x00\x00" => {
                    decompress(&compressed_key_block_bytes[8..], &mut key_block);
                    let mut record_index_list =
                        RecordIndex::list_from_one_key_block_bytes(&key_block);
                    whole_index_list.append(&mut record_index_list);
                }
                _ => {
                    todo!()
                }
            }

            i += one_codec_info.compressed_size;
        }
        return whole_index_list;
    }

    /// 将一个 key block bytes 解析成 Vec<RecordIndex>
    fn list_from_one_key_block_bytes(key_block: &Vec<u8>) -> Vec<RecordIndex> {
        let num_width: usize = 8;
        let mut key_start = 0; //一个RecordIndex的起点
        let mut key_end = 0; //一个RecordIndex的终点

        let delimiter = b"\x00";
        let delimiter_width = 1;
        let mut record_index_list: Vec<RecordIndex> = vec![];
        while key_start < key_block.len() {
            let slice = &key_block[key_start..(key_start + num_width)];
            let index = u64::from_be_bytes(slice.try_into().unwrap());
            let mut text_start = key_start + num_width;
            while text_start < key_block.len() {
                if &key_block[text_start..(text_start + delimiter_width)] == delimiter {
                    key_end = text_start;
                    break;
                }
                text_start += delimiter_width;
            }
            let text = std::str::from_utf8(&key_block[(key_start + num_width)..(key_end)])
                .unwrap()
                .to_string();
            key_start = key_end + delimiter_width;

            record_index_list.push(RecordIndex { index, text });
        }
        return record_index_list;
    }
}
