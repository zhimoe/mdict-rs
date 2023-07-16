use std::collections::HashMap;

use log::info;
use regex::Regex;

use crate::unpack::string_from_utf16_le;

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
        let header = &header_bytes[..header_bytes.len() - 2];
        let header_txt =
            string_from_utf16_le(&header).expect("convert slice to utf16 string in little endian");
        info!("header_text {}", &header_txt);

        let mut _header_map: HashMap<String, String> = HashMap::new();
        let re = Regex::new(r#"(\w+)="(.*?)""#).unwrap();
        for cap in re.captures_iter(header_txt.as_str()) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            _header_map.insert(key, value);
        }
        info!("header_map {:?}", &_header_map);
        return Header {
            engine_version: _header_map
                .get("GeneratedByEngineVersion")
                .unwrap()
                .parse::<f32>()
                .unwrap(),
            format: _header_map["Format"].clone(),
            key_case_sensitive: _header_map["KeyCaseSensitive"] == "Yes",
            strip_key: _header_map["StripKey"] == "Yes",
            encrypted: _header_map["Encrypted"].clone(),
            register_by: _header_map["RegisterBy"].clone(),
            encoding: _header_map["Encoding"].clone(),
            creation_date: _header_map["CreationDate"].clone(),
            compact: _header_map["Compact"] == "Yes",
            left2right: _header_map["Left2Right"] == "Yes",
            datasource_format: _header_map["DataSourceFormat"].clone(),
            stylesheet: _header_map["StyleSheet"].clone(),
            key_block_offset: 0,
            record_block_offset: 0,
        };
    }
}
