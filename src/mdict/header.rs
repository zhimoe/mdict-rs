use std::collections::HashMap;
use regex::Regex;
use crate::unpack::utf16_string_le;

/// Header raw分为三段:
/// bytes len of dict info(4 bytes int),
/// dict info(bytes): 即下面的Header
/// adler32 checksum(4 bytes int)
#[derive(Debug, Default)]
pub struct Header {
    pub engine_version: f32,
    // important
    pub format: String,
    pub key_case_sensitive: bool,
    pub strip_key: bool,
    /**
     * encryption flag!
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block
     */
    pub encrypted: String,
    pub register_by: String,
    pub encoding: String,
    pub creation_date: String,
    pub compact: bool,
    pub left2right: bool,
    pub datasource_format: String,
    pub stylesheet: String,
    pub key_block_offset: u64,
    pub record_block_offset: u64,
}

impl Header {
    /// build header info from bytes
    pub fn build_from_bytes(header_bytes: Vec<u8>) -> Self {
        // header text in utf-16 encoding ending with '\x00\x00'
        let (header, _end) = header_bytes.split_at(header_bytes.len() - 2);
        let header_txt = utf16_string_le(&header).expect("convert slice to utf16 string in little endian");

        let mut _header_map: HashMap<String, String> = HashMap::new();
        let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
        let cap_matches = re.captures_iter(header_txt.as_str());
        for cap in cap_matches {
            _header_map.insert(cap.get(1).unwrap().as_str().to_string(),
                               cap.get(2).unwrap().as_str().to_string());
        }
        println!("the header map {:?}", &_header_map);

        return Header {
            engine_version: _header_map.get("GeneratedByEngineVersion").unwrap().parse::<f32>().unwrap(),
            format: _header_map.get("Format").unwrap().to_string(),
            key_case_sensitive: _header_map.get("KeyCaseSensitive").unwrap() == "Yes",
            strip_key: _header_map.get("StripKey").unwrap() == "Yes",
            encrypted: _header_map.get("Encrypted").unwrap().to_string(),
            register_by: _header_map.get("RegisterBy").unwrap().to_string(),
            encoding: _header_map.get("Encoding").unwrap().to_string(),
            creation_date: _header_map.get("Encoding").unwrap().to_string(),
            compact: _header_map.get("Compact").unwrap() == "Yes",
            left2right: _header_map.get("Left2Right").unwrap() == "Yes",
            datasource_format: _header_map.get("DataSourceFormat").unwrap().to_string(),
            stylesheet: _header_map.get("StyleSheet").unwrap().to_string(),
            key_block_offset: 0,
            record_block_offset: 0,
        };
    }
}
