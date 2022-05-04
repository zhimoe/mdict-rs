use std::collections::HashMap;
use regex::Regex;
use crate::unpack::utf16_le_string;

/// Header raw分为三段:
/// bytes len of dict info(4 bytes int),
/// dict info(bytes): 即下面的Header
/// adler32 checksum(4 bytes int)
#[derive(Debug, Default)]
pub struct Header {
    pub file: String,
    // important
    pub gen_version: f32,
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
    pub fn file(&mut self, file: &str) -> &mut Self {
        self.file = file.to_string();
        self
    }
    pub fn gen_version(&mut self, gen_version: f32) -> &mut Self {
        self.gen_version = gen_version;
        self
    }
    pub fn format(&mut self, format: &str) -> &mut Self {
        self.format = format.to_string();
        self
    }
    pub fn key_case_sensitive(&mut self, key_case_sensitive: bool) -> &mut Self {
        self.key_case_sensitive = key_case_sensitive;
        self
    }
    pub fn strip_key(&mut self, strip_key: bool) -> &mut Self {
        self.strip_key = strip_key;
        self
    }
    pub fn encrypted(&mut self, encrypted: &str) -> &mut Self {
        self.encrypted = encrypted.to_string();
        self
    }
    pub fn register_by(&mut self, register_by: &str) -> &mut Self {
        self.register_by = register_by.to_string();
        self
    }
    pub fn encoding(&mut self, encoding: &str) -> &mut Self {
        self.encoding = encoding.to_string();
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
    pub fn datasource_format(&mut self, datasource_format: &str) -> &mut Self {
        self.datasource_format = datasource_format.to_string();
        self
    }
    pub fn stylesheet(&mut self, stylesheet: &str) -> &mut Self {
        self.stylesheet = stylesheet.to_string();
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

    /// extract header info from dict info bytes
    pub fn extract_from_dict_bytes(&mut self, header_bytes: Vec<u8>) -> &mut Self {
        // header text in utf-16 encoding ending with '\x00\x00'
        let (header, _end) = header_bytes.split_at(header_bytes.len() - 2);
        let header_txt = utf16_le_string(&header).expect("convert slice to utf16 string in little endian");

        let mut _header_map = HashMap::new();
        let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
        let cap_matches = re.captures_iter(header_txt.as_str());
        for cap in cap_matches {
            _header_map.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
        }
        if let Some(v) = _header_map.get(&"GeneratedByEngineVersion") {
            self.gen_version(v.parse::<f32>().unwrap());
        }
        if let Some(f) = _header_map.get(&"Format") {
            self.format(f);
        }
        if let Some(k) = _header_map.get(&"KeyCaseSensitive") {
            self.key_case_sensitive(k == &"Yes");
        }
        if let Some(s) = _header_map.get(&"StripKey") {
            self.strip_key(s == &"Yes");
        }
        if let Some(en) = _header_map.get(&"Encrypted") {
            self.encrypted(en);
        }
        if let Some(r) = _header_map.get(&"RegisterBy") {
            self.register_by(r);
        }
        if let Some(encoding) = _header_map.get(&"Encoding") {
            self.encoding(encoding);
        }
        if let Some(d) = _header_map.get(&"DataSourceFormat") {
            self.datasource_format(d);
        }
        if let Some(s) = _header_map.get(&"StyleSheet") {
            self.stylesheet(s);
        }
        if let Some(c) = _header_map.get(&"Compact") { //or Compat
            self.compact(c == &"Yes");
        }
        if let Some(l) = _header_map.get(&"Left2Right") {
            self.left2right(l == &"Yes");
        }

        self
    }
}
