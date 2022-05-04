use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use flate2::write::ZlibDecoder;
use ripemd128::{Digest, Ripemd128};

use crate::checksum::adler32_checksum;
use crate::number::{NumberBytes, read_number};
use crate::unpack::{Endian, unpack_u16, unpack_u32, unpack_u64};

use super::header::{Header};
use super::key::KeyIndex;
use super::record::RecordIndex;

pub struct Mdx {
    pub filename: String,
    pub header: Header,
    pub passcode: String,
    pub version: f32,
    // important, for bytes to number unpack
    pub num_entries: u64,
    pub number_width: i32,
    pub num_key_blocks: u64,
    pub num_record_blocks: u64,
    pub keys: Vec<KeyIndex>,
    pub records: Vec<RecordIndex>,
}

impl Mdx {
    pub(crate) fn new(file: &str) -> Mdx {
        let mut reader = BufReader::new(File::open(&file).unwrap());
        let mut _bytes4 = [0; 4];
        reader.read_exact(&mut _bytes4).expect("read_exact 4 bytes error"); // read exactly 4 bytes
        let header_len = unpack_u32(&_bytes4, Endian::BE);

        let mut header_bytes = vec![0; header_len as usize];
        reader.read_exact(&mut header_bytes).expect("read header bytes of dict info error");

        // reade 4 bytes: adler32 checksum of header, in little endian
        let mut adler32_bytes = [0; 4];
        reader.read_exact(&mut adler32_bytes).expect("read adler32_bytes error");
        if !adler32_checksum(&header_bytes, &adler32_bytes, Endian::LE) {
            panic!("unrecognized format");
        } else {
            println!("header bytes adler32_checksum success")
        }

        let mut header = Header::default();
        header.file(file);
        header.key_block_offset(reader.seek(SeekFrom::Current(0)).expect("Could not get current file position!"));
        header.extract_from_dict_bytes(header_bytes);

        // parser key block
        let number_width = if header.gen_version >= 2.0 { 8 } else { 4 };
        let meta_bytes_size = if header.gen_version >= 2.0 { 8 * 5 } else { 4 * 4 }; // 看结构图https://bitbucket.org/xwang/mdict-analysis/src/master/MDX.svg
        let mut key_block_info_meta_bytes = vec![0; meta_bytes_size];
        reader.read_exact(&mut key_block_info_meta_bytes).expect("read key_block_info_meta_bytes error");

        let mut meta_number_bytes = NumberBytes::new(&key_block_info_meta_bytes);
        let key_blocks_num = meta_number_bytes.read_number(number_width).unwrap();
        let entries_num = meta_number_bytes.read_number(number_width).unwrap();
        println!("num entries={}", entries_num);
        if header.gen_version >= 2.0 {
            let _key_block_info_decompress_size = meta_number_bytes.read_number(number_width);
            println!("from meta_number_bytes _key_block_info_decompress_size={}", _key_block_info_decompress_size.unwrap());
        }
        let key_block_info_bytes_size = meta_number_bytes.read_number(number_width).unwrap();
        let key_block_bytes_size = meta_number_bytes.read_number(number_width).unwrap();

        // reade 4 bytes: adler32 checksum of key block info, in big endian only version >= 2.0
        if header.gen_version >= 2.0 {
            let mut adler32_bytes = [0; 4];
            reader.read_exact(&mut adler32_bytes).expect("read_exact error");

            if !adler32_checksum(&key_block_info_meta_bytes, &adler32_bytes, Endian::BE) {
                panic!("key block info adler32 checksum error, unrecognized format");
            } else {
                println!("key block info adler32 checksum success")
            }
        }

        let mut key_block_info_bytes = vec![0; key_block_info_bytes_size as usize];
        reader.read_exact(&mut key_block_info_bytes).expect("read_exact error");

        let mut key_block_list_bytes = vec![0; key_block_bytes_size as usize];
        reader.read_exact(&mut key_block_list_bytes).expect("read_exact error");

        let current_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current file position!");
        header.record_block_offset(current_pos);

        let key_block_comp_decomp_size_list = decode_key_block_info(&key_block_info_bytes, &header);//(key_block_compressed_size, key_block_decompressed_size)
        println!("key_block_comp_decomp_size_list size={}", &key_block_comp_decomp_size_list.len());
        let key_list = decode_key_block(&key_block_list_bytes, &key_block_comp_decomp_size_list);
        println!("key index size={}", &key_list.len());

        //parse record block
        let num_record_blocks = read_number(&mut reader, number_width);
        let num_entries = read_number(&mut reader, number_width);
        let record_block_info_size = read_number(&mut reader, number_width);
        let _record_block_size = read_number(&mut reader, number_width);
        let mut record_block_comp_decomp_size_list: Vec<(usize, usize)> = vec![];
        let mut size_counter = 0;
        // read all record_block_info bytes
        for _i in 0..num_record_blocks {
            let compressed_size = read_number(&mut reader, number_width);
            let decompressed_size = read_number(&mut reader, number_width);
            record_block_comp_decomp_size_list.push((compressed_size, decompressed_size));
            size_counter += number_width * 2
        }
        assert_eq!(size_counter, record_block_info_size);

        // start read record block, decompress it
        let mut record_list: Vec<RecordIndex> = vec![]; // important!
        let mut _record_block_bytes_size_counter = 0;
        let mut i: usize = 0;
        let mut offset: usize = 0;

        for (c_size, d_size) in record_block_comp_decomp_size_list {
            let cur_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current file position!");
            let mut record_block_compressed: Vec<u8> = vec![0; c_size];
            reader.read_exact(&mut record_block_compressed).expect("read_exact error");

            let (_record_block_decompressed, block_typ) = decompress_record_block_bytes(&mut record_block_compressed);

            // split record block into record according to the offset info from key block
            while i < key_list.len() {
                let key_index = &key_list[i];
                let start = key_index.key_id as usize;
                if start - offset >= d_size {
                    break;
                }
                let record_end: usize;
                if i < key_list.len() - 1 {
                    record_end = key_list[i + 1].key_id as usize;
                } else {
                    record_end = d_size + offset;
                }
                let idx = RecordIndex {
                    key_text: key_index.key_text.to_string(),
                    file_pos: cur_pos as u32,
                    compressed_size: c_size as u32,
                    decompressed_size: d_size as u32,
                    record_block_type: block_typ as u32,
                    record_start: key_index.key_id.clone() as u32,
                    record_end: record_end as u32,
                    offset: offset as u32,
                };
                i += 1;

                // let mut record = &record_block_decompressed[(start - offset)..(record_end - offset)];
                // let content = String::from_utf8_lossy(record);
                record_list.push(idx)
            }
            offset += d_size;
            _record_block_bytes_size_counter += c_size;
        }

        let version = header.gen_version;
        Mdx {
            filename: file.to_string(),
            header,
            passcode: "".to_string(),
            version,
            number_width: number_width as i32,
            num_entries: num_entries as u64,
            num_key_blocks: key_blocks_num,
            num_record_blocks: 122,
            keys: key_list,
            records: record_list,
        }
    }


    // util function, extract word definitions from bytes
    pub fn extract_definition(record_block_compressed: &mut Vec<u8>, record_start: usize, record_end: usize, offset: usize) -> String {
        let record_block_type = &record_block_compressed[0..4];
        let adler32_bytes = &record_block_compressed[4..8];
        let mut record_block_decompressed = Vec::new();
        assert_eq!(b"\x02\x00\x00\x00", record_block_type);

        let mut z = ZlibDecoder::new(record_block_decompressed);
        z.write_all(&record_block_compressed[8..]).unwrap();
        record_block_decompressed = z.finish().unwrap();
        if !adler32_checksum(&record_block_decompressed, &adler32_bytes, Endian::BE) {
            panic!("record block adler32 checksum failed");
        }
        let s = record_start - offset;
        let e = record_end - offset;
        let record = &record_block_decompressed[s..e];
        let def = String::from_utf8_lossy(record);
        def.to_string()
    }
}

fn decompress_record_block_bytes(record_block_compressed: &mut Vec<u8>) -> (Vec<u8>, i32) {
    let record_block_type = &record_block_compressed[0..4];
    let adler32_bytes = &record_block_compressed[4..8];
    let mut record_block_decompressed = Vec::new();
    let mut _type = 2;
    match record_block_type {
        b"\x02\x00\x00\x00" => {
            _type = 2;
            let mut z = ZlibDecoder::new(record_block_decompressed);
            z.write_all(&record_block_compressed[8..]).unwrap();
            record_block_decompressed = z.finish().unwrap();
            if !adler32_checksum(&record_block_decompressed, &adler32_bytes, Endian::BE) {
                panic!("record block adler32 checksum failed");
            }
        }
        b"\x01\x00\x00\x00" => { println!("\x01\x00\x00\x00") }
        b"\x00\x00\x00\x00" => { println!("\x00\x00\x00\x00") }
        _ => {}
    }
    (record_block_decompressed, _type)
}

/// return <compressed size, decompressed size>
pub fn decode_key_block_info(key_block_info_compressed: &Vec<u8>, header: &Header) -> Vec<(usize, usize)> {
    let first4 = &key_block_info_compressed[0..4];
    let mut adler32_bytes = &key_block_info_compressed[4..8];
    let data = &key_block_info_compressed[8..];
    let mut decrypted_compressed_bytes = vec![0; data.len()];
    let mut decompressed_key_block_info_bytes = Vec::new();
    if header.gen_version >= 2.0 {
        assert_eq!(b"\x02\x00\x00\x00", first4);
        let encrypted = header.encrypted.parse::<u32>().unwrap();
        // 如果key block info部分编码了,先解码
        if &encrypted & 0x02 == 0x02 {
            println!("key block info bytes encrypted");
            let key = get_key_block_info_decrypt_key(&mut adler32_bytes);
            let mut previous: u8 = 0x36;
            for i in 0..data.len() {
                let mut t = (data[i] >> 4 | data[i] << 4) & 0xff;
                t = t ^ previous ^ (i & 0xff) as u8 ^ key[i % key.len()];
                previous = data[i].clone();
                decrypted_compressed_bytes[i] = t;
            }
        } else {
            println!("key block info bytes not encrypted");
            decrypted_compressed_bytes = Vec::from(data.clone());
        }

        let mut z = ZlibDecoder::new(decompressed_key_block_info_bytes);
        z.write_all(decrypted_compressed_bytes.as_ref()).unwrap();
        decompressed_key_block_info_bytes = z.finish().unwrap();

        //data now is decrypted, then decompress
        if !adler32_checksum(&decompressed_key_block_info_bytes, &adler32_bytes, Endian::BE) {
            panic!("key_block_info adler32 checksum failed!")
        }
    }

    //start decode
    // let mut key_block_info_list = vec![];
    let mut _num_enteries = 0 as u64;
    let mut byte_width = 1;
    let mut text_term = 0;
    if header.gen_version >= 2.0 {
        byte_width = 2;
        text_term = 1;
    }
    let num_width = 8;
    let mut i = 0;
    let mut key_block_info_list: Vec<(usize, usize)> = vec![];
    while i < decompressed_key_block_info_bytes.len() {
        _num_enteries += unpack_u64(&decompressed_key_block_info_bytes[i..(i + num_width)], Endian::BE);
        i += num_width;
        let text_head_size = unpack_u16(&decompressed_key_block_info_bytes[i..(i + byte_width)], Endian::BE);
        i += byte_width;
        i += (text_head_size + text_term) as usize;
        let text_tail_size = unpack_u16(&decompressed_key_block_info_bytes[i..(i + byte_width)], Endian::BE);
        i += byte_width;
        i += (text_tail_size + text_term) as usize;

        let key_block_compressed_size = unpack_u64(&decompressed_key_block_info_bytes[i..(i + num_width)], Endian::BE);
        i += num_width;
        let key_block_decompressed_size = unpack_u64(&decompressed_key_block_info_bytes[i..(i + num_width)], Endian::BE);
        i += num_width;
        key_block_info_list.push((key_block_compressed_size as usize, key_block_decompressed_size as usize))
    }
    return key_block_info_list;
}


fn get_key_block_info_decrypt_key(adler32_bytes: &mut &[u8]) -> Vec<u8> {
    let fix: Vec<u8> = vec![0x95, 0x36, 0x00, 0x00];//0x3695 in little endian
    // create a RIPEMD-128 hasher instance
    let mut hasher = Ripemd128::new();
    hasher.input([&adler32_bytes, &fix[..]].concat());
    // acquire hash digest in the form of GenericArray,
    // which in this case is equivalent to [u8; 16]
    let ga = hasher.result();
    ga.as_slice().iter().cloned().collect()
}

fn decode_key_block(all_key_block_bytes: &Vec<u8>, key_block_info_list: &Vec<(usize, usize)>) -> Vec<KeyIndex> {
    let mut key_list: Vec<KeyIndex> = vec![];
    let mut i = 0;
    let mut end: usize = i;
    for (key_block_compressed_size, _key_block_decompressed_size) in key_block_info_list {
        //  key_block_decompressed_size is used when key_block_type=b"\x02\x00\x00\x00"
        let start = i;
        end += *key_block_compressed_size;

        let one_key_block_bytes = &all_key_block_bytes[start as usize..end as usize];

        let key_block_type = &one_key_block_bytes[0..4];
        let _adler32_bytes = &one_key_block_bytes[4..8];
        let mut key_block = Vec::new();
        match key_block_type {
            b"\x02\x00\x00\x00" => {
                let mut z = ZlibDecoder::new(key_block);
                z.write_all(&one_key_block_bytes[8..]).unwrap();
                key_block = z.finish().unwrap();
                split_key_block(&key_block, &mut key_list);
            }
            _ => {}
        }

        i += *key_block_compressed_size as usize;
    }
    return key_list;
}


/// 将一个key block 中的多个 key_id,key_text解析出来得到一个Vec<KeyIndex>
fn split_key_block(key_block: &Vec<u8>, key_index_list: &mut Vec<KeyIndex>) {
    let num_width: usize = 8;
    let mut key_start = 0; //一个keyIndex的起点
    let mut key_end = 0;//一个keyIndex的终点

    let delimiter = b"\x00";
    let delimiter_width = 1;

    while key_start < key_block.len() {
        let slice = &key_block[key_start..(key_start + num_width)];
        let key_id = unpack_u64(slice, Endian::BE);

        let mut text_start = key_start + num_width;
        while text_start < key_block.len() {
            if &key_block[text_start..(text_start + delimiter_width)] == delimiter {
                key_end = text_start;
                break;
            }
            text_start += delimiter_width;
        }
        let key_text = std::str::from_utf8(&key_block[(key_start + num_width)..(key_end)]).unwrap().to_string();
        key_start = key_end + delimiter_width;
        key_index_list.push(KeyIndex {
            key_id,
            key_text,
        });
    }
}