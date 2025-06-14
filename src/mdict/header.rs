use std::collections::HashMap;

use adler32::adler32;
use encoding::{all::UTF_16LE, Encoding};
use nom::multi::length_data;
use nom::number::complete::{be_u32, le_u32};
use nom::{IResult, Parser};
use regex::Regex;
use tracing::info;

#[derive(Debug)]
pub enum Version {
    V1,
    V2,
}

/// mdx头部信息
#[derive(Debug)]
pub struct Header {
    // 牛津8/汉语词典3/朗文4都是 V2
    pub version: Version,
    /**
     * encryption flag "0"-no encryption, "1"-encrypt record block, "2"-encrypt key info block
     * e.g., 牛津8/汉语词典3 "0" 朗文4 "2"
     */
    pub encrypted: String,
    // record bytes encoding, e.g. "UTF-8"
    pub encoding: String,
}

pub fn parse_header(data: &[u8]) -> IResult<&[u8], Header> {
    // length_data(be_u32) 先读取一个be_u32 number,然后根据number读取对应长度bytes
    let (data, (header_buf, checksum)) = (length_data(be_u32), le_u32).parse(data)?;
    // &[8] 实现Read接口
    assert_eq!(adler32(header_buf).unwrap(), checksum);
    // string from utf_16le encoding
    let info = UTF_16LE
        .decode(header_buf, encoding::DecoderTrap::Strict)
        .unwrap();

    let re = Regex::new(r#"(\w+)="((.|\r\n|[\r\n])*?)""#).unwrap();
    let mut attrs = HashMap::new();
    for cap in re.captures_iter(info.as_str()) {
        attrs.insert(cap[1].to_string(), cap[2].to_string());
    }

    info!(">>>the header content: {:?}", &attrs);

    let version = attrs
        .get("GeneratedByEngineVersion")
        .unwrap()
        .trim()
        .chars()
        .next()
        .unwrap()
        .to_digit(10)
        .unwrap() as u8;

    let version = match version {
        1 => Version::V1,
        2 => Version::V2,
        _ => panic!("unsupported mdx engine version!, {}", &version),
    };

    // "0" "2" "3"
    let encrypted = attrs.get("Encrypted").unwrap().to_string();

    // "UTF-8"
    let encoding = attrs.get("Encoding").unwrap().to_string();

    Ok((
        data,
        Header {
            version,
            encrypted,
            encoding,
        },
    ))
}
