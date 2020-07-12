extern crate ripemd128;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use flate2::write::ZlibDecoder;
use regex::Regex;
use ripemd128::{Digest, Ripemd128};

use crate::checksum::adler32_checksum;
use crate::number::{NumberBytes, read_number};
use crate::unpack::{Endian, unpack_u16, unpack_u32, unpack_u64, utf16_le_string};


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

#[derive(Debug)]
pub struct Header {
    pub file: String,
    pub genversion: f32,
    pub format: String,
    pub keycasesensitive: bool,
    pub stripkey: bool,
    /**
     * encryption flag
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block
     */
    pub encrypted: String,
    pub registerby: String,
    pub encoding: String,
    pub creationdate: String,
    pub compact: bool,
    pub left2right: bool,
    pub datasourceformat: String,
    pub stylesheet: String,
    pub key_block_offset: u64,
    pub record_block_offset: u64,
}

#[derive(Debug, Default)]
pub struct HeaderBuilder {
    pub file: String,
    pub genversion: f32,
    pub format: String,
    pub keycasesensitive: bool,
    pub stripkey: bool,
    pub encrypted: String,
    pub registerby: String,
    pub encoding: String,
    pub compact: bool,
    pub left2right: bool,
    pub datasourceformat: String,
    pub stylesheet: String,
    pub key_block_offset: u64,
    pub record_block_offset: u64,
}

impl HeaderBuilder {
    pub fn file(&mut self, file: String) -> &mut Self {
        self.file = file;
        self
    }
    pub fn genversion(&mut self, genversion: f32) -> &mut Self {
        self.genversion = genversion;
        self
    }
    pub fn format(&mut self, format: String) -> &mut Self {
        self.format = format;
        self
    }
    pub fn keycasesensitive(&mut self, keycasesensitive: bool) -> &mut Self {
        self.keycasesensitive = keycasesensitive;
        self
    }
    pub fn stripkey(&mut self, stripkey: bool) -> &mut Self {
        self.stripkey = stripkey;
        self
    }
    pub fn encrypted(&mut self, encrypted: String) -> &mut Self {
        self.encrypted = encrypted;
        self
    }
    pub fn registerby(&mut self, registerby: String) -> &mut Self {
        self.registerby = registerby;
        self
    }
    pub fn encoding(&mut self, encoding: String) -> &mut Self {
        self.encoding = encoding;
        self
    }
    pub fn compact(&mut self, compact: bool) -> &mut Self {
        self.compact = compact;
        self
    }
    pub fn left2right(&mut self, left2right: bool) -> &mut Self {
        self.left2right = left2right;
        self
    }
    pub fn datasourceformat(&mut self, datasourceformat: String) -> &mut Self {
        self.datasourceformat = datasourceformat;
        self
    }
    pub fn stylesheet(&mut self, stylesheet: String) -> &mut Self {
        self.stylesheet = stylesheet;
        self
    }
    pub fn key_block_offset(&mut self, key_block_offset: u64) -> &mut Self {
        self.key_block_offset = key_block_offset;
        self
    }
    pub fn record_block_offset(&mut self, record_block_offset: u64) -> &mut Self {
        self.record_block_offset = record_block_offset;
        self
    }
    pub fn build(&self) -> Header {
        Header {
            file: self.file.to_owned(),
            genversion: self.genversion,
            format: self.format.to_owned(),
            keycasesensitive: self.keycasesensitive,
            stripkey: self.stripkey,
            encrypted: self.encrypted.to_owned(),
            registerby: self.registerby.to_owned(),
            encoding: self.encoding.to_owned(),
            creationdate: "".to_string(),
            compact: self.compact,
            left2right: self.left2right,
            datasourceformat: self.datasourceformat.to_owned(),
            stylesheet: self.stylesheet.to_owned(),
            key_block_offset: self.key_block_offset,
            record_block_offset: self.record_block_offset,
        }
    }
}

pub struct KeyIndex {
    pub key_id: u64,
    pub key_text: String,
}

pub struct Mdx {
    pub filename: String,
    pub header: Header,
    pub passcode: String,
    pub version: f32,
    // important, for bytes to number unpack
    pub number_width: i32,
    pub num_entries: u64,
    pub num_key_blocks: u64,
    pub num_record_blocks: u64,
    pub keys: Vec<KeyIndex>,
    pub records: Vec<RecordIndex>,
}

impl Mdx {
    pub(crate) fn new(file: &str) -> Mdx {
        let mut hb = HeaderBuilder::default();
        hb.file(file.to_string());
        let mut reader = BufReader::new(File::open(&file).unwrap());
        let mut _bytes4 = [0; 4];
        reader.read_exact(&mut _bytes4).expect("read_exact error"); // read exactly 4 bytes
        let header_len = unpack_u32(&_bytes4, Endian::BE);

        let mut header_bytes = vec![0; header_len as usize];
        reader.read_exact(&mut header_bytes).expect("read header bytes error");

        // reade 4 bytes: adler32 checksum of header, in little endian
        let mut adler32_bytes = [0; 4];
        reader.read_exact(&mut adler32_bytes).expect("read adler32_bytes error");

        if !adler32_checksum(&header_bytes, &adler32_bytes, Endian::LE) {
            panic!("unrecognized format");
        } else {
            println!("header bytes adler32_checksum success")
        }

        let current_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current file position!");
        hb.key_block_offset(current_pos);

        // header text in utf-16 encoding ending with '\x00\x00'
        let (header, _end) = header_bytes.split_at(header_bytes.len() - 2);
        let header_txt = utf16_le_string(&header).expect("convert slice to utf16 string in little endian");

        extract_header(&mut hb, header_txt);

        // key block info
        let _num_width = if hb.genversion >= 2.0 { 8 } else { 4 };
        let meta_bytes_size = if hb.genversion >= 2.0 { 8 * 5 } else { 4 * 4 };
        let mut key_block_info_meta_bytes = vec![0; meta_bytes_size];
        reader.read_exact(&mut key_block_info_meta_bytes).expect("read key_block_info_meta_bytes error");

        let mut nb = NumberBytes::new(&key_block_info_meta_bytes);
        let num_key_blocks = nb.read_number(_num_width).unwrap();
        let _num_entries = nb.read_number(_num_width).unwrap();
        println!("num entries={}", _num_entries);
        if hb.genversion >= 2.0 {
            let _key_block_info_decompress_size = nb.read_number(_num_width);
        }
        let key_block_info_size = nb.read_number(_num_width).unwrap();
        let key_block_size = nb.read_number(_num_width).unwrap();

        // reade 4 bytes: adler32 checksum of key block info, in big endian
        let mut adler32_bytes = [0; 4];
        reader.read_exact(&mut adler32_bytes).expect("read_exact error");

        if !adler32_checksum(&key_block_info_meta_bytes, &adler32_bytes, Endian::BE) {
            panic!("key block info adler32_checksum error, unrecognized format");
        } else {
            println!("key block info adler32_checksum success")
        }

        let mut key_block_info_bytes = vec![0; key_block_info_size as usize];
        reader.read_exact(&mut key_block_info_bytes).expect("read_exact error");

        let mut key_block_bytes = vec![0; key_block_size as usize];
        reader.read_exact(&mut key_block_bytes).expect("read_exact error");

        let current_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current file position!");
        hb.record_block_offset(current_pos);

        let header = hb.build();
        let key_block_comp_decomp_size_list = decode_key_block_info(&key_block_info_bytes, &header);//(key_block_compressed_size, key_block_decompressed_size)
        println!("key_block_comp_decomp_size_list size={}", &key_block_comp_decomp_size_list.len());
        let key_list = decode_key_block(&key_block_bytes, &key_block_comp_decomp_size_list);
        println!("key size={}", &key_list.len());
        //parse record block
        let num_record_blocks = read_number(&mut reader, _num_width);
        let num_entries = read_number(&mut reader, _num_width);
        let record_block_info_size = read_number(&mut reader, _num_width);
        let _record_block_size = read_number(&mut reader, _num_width);
        let mut record_block_comp_decomp_size_list: Vec<(usize, usize)> = vec![];
        let mut size_counter = 0;
        // read all record_block_info bytes
        for _i in 0..num_record_blocks {
            let compressed_size = read_number(&mut reader, _num_width);
            let decompressed_size = read_number(&mut reader, _num_width);
            record_block_comp_decomp_size_list.push((compressed_size, decompressed_size));
            size_counter += _num_width * 2
        }
        assert_eq!(size_counter, record_block_info_size);

        // start read record block, decompress it
        let mut record_list: Vec<RecordIndex> = vec![]; // important!
        let mut _record_block_bytes_size_counter = 0;
        let mut i: usize = 0;
        let mut offset: usize = 0;

        let mut record_block_counter = 0;
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
                let mut record_end: usize = 0;
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
            record_block_counter += 1;
        }

        let version = header.genversion;
        Mdx {
            filename: file.to_string(),
            header,
            passcode: "".to_string(),
            version: version,
            number_width: _num_width as i32,
            num_entries: num_entries as u64,
            num_key_blocks: num_key_blocks,
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

fn extract_header(hb: &mut HeaderBuilder, header_txt: String) {
    let mut _header_map = HashMap::new();
    let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
    let cap_matches = re.captures_iter(header_txt.as_str());
    for cap in cap_matches {
        _header_map.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
    }

    if let Some(v) = _header_map.get(&"GeneratedByEngineVersion") {
        hb.genversion(v.parse::<f32>().unwrap());
    }

    if let Some(f) = _header_map.get(&"Format") {
        hb.format(f.to_string());
    }
    if let Some(k) = _header_map.get(&"KeyCaseSensitive") {
        if k == &"Yes" {
            hb.keycasesensitive(true);
        } else {
            hb.keycasesensitive(false);
        }
    }
    if let Some(s) = _header_map.get(&"StripKey") {
        if s == &"Yes" {
            hb.stripkey(true);
        } else {
            hb.stripkey(false);
        }
    }
    if let Some(en) = _header_map.get(&"Encrypted") {
        hb.encrypted(en.to_string());
    }
    if let Some(r) = _header_map.get(&"RegisterBy") {
        hb.registerby(r.to_string());
    }
    if let Some(encoding) = _header_map.get(&"Encoding") {
        hb.encoding(encoding.to_string());
    }
    if let Some(d) = _header_map.get(&"DataSourceFormat") {
        hb.datasourceformat(d.to_string());
    }
    if let Some(s) = _header_map.get(&"StyleSheet") {
        hb.stylesheet(s.to_string());
    }
    if let Some(c) = _header_map.get(&"Compact") { //or Compat
        if c == &"Yes" {
            hb.compact(true);
        } else {
            hb.compact(false);
        }
    }
    if let Some(l) = _header_map.get(&"Left2Right") {
        if l == &"Yes" {
            hb.left2right(true);
        } else {
            hb.left2right(false);
        }
    }
}


pub fn decode_key_block_info(key_block_info_compressed: &Vec<u8>, header: &Header) -> Vec<(usize, usize)> {

    let first4 = &key_block_info_compressed[0..4];
    let mut adler32_bytes = &key_block_info_compressed[4..8];
    let data = &key_block_info_compressed[8..];
    let mut decrypt_bytes = vec![0; data.len()];
    let mut key_block_info_bytes = Vec::new();
    if header.genversion >= 2.0 {
        assert!(b"\x02\x00\x00\x00" == first4);
        let encrypted = header.encrypted.parse::<u32>().unwrap();
        if &encrypted & 0x02 == 0x02 {
            let key = get_key_block_info_decrypt_key(&mut adler32_bytes);
            let mut previous: u8 = 0x36;
            for i in 0..data.len() {
                let mut t = (data[i] >> 4 | data[i] << 4) & 0xff;
                t = t ^ previous ^ (i & 0xff) as u8 ^ key[i % key.len()];
                previous = data[i].clone();
                decrypt_bytes[i] = t;
            }
        } else {
            println!("header encrypted={}", &encrypted);
            decrypt_bytes = Vec::from(data.clone());
        }

        let mut z = ZlibDecoder::new(key_block_info_bytes);
        z.write_all(decrypt_bytes.as_ref()).unwrap();
        key_block_info_bytes = z.finish().unwrap();

        //data now is decrypted, then decompress
        if !adler32_checksum(&key_block_info_bytes, &adler32_bytes, Endian::BE) {
            panic!("key_block_info adler32 checksum failed!")
        }
    }

    //start decode
    // let mut key_block_info_list = vec![];
    let mut _num_enteries = 0 as u64;
    let mut byte_width = 1;
    let mut text_term = 0;
    if header.genversion >= 2.0 {
        byte_width = 2;
        text_term = 1;
    }
    let num_width = 8;
    let mut i = 0;
    let mut key_block_info_list: Vec<(usize, usize)> = vec![];
    while i < key_block_info_bytes.len() {
        _num_enteries += unpack_u64(&key_block_info_bytes[i..(i + num_width)], Endian::BE);
        i += num_width;
        let text_head_size = unpack_u16(&key_block_info_bytes[i..(i + byte_width)], Endian::BE);
        i += byte_width;
        i += (text_head_size + text_term) as usize;
        let text_tail_size = unpack_u16(&key_block_info_bytes[i..(i + byte_width)], Endian::BE);
        i += byte_width; //todo:
        i += (text_tail_size + text_term) as usize;

        let key_block_compressed_size = unpack_u64(&key_block_info_bytes[i..(i + num_width)], Endian::BE);
        i += num_width;
        let key_block_decompressed_size = unpack_u64(&key_block_info_bytes[i..(i + num_width)], Endian::BE);
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
    let mut start = 0;
    let mut end = i;
    for (key_block_compressed_size, _key_block_decompressed_size) in key_block_info_list { //  key_block_decompressed_size is used when key_block_type=b"\x02\x00\x00\x00"
        start = i;
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
    let mut text_start = 0;

    while key_start < key_block.len() {
        let slice = &key_block[key_start..(key_start + num_width)];
        let key_id = unpack_u64(slice, Endian::BE);

        text_start = key_start + num_width;
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