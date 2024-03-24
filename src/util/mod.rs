use nom::IResult;
use nom::number::complete::{be_u16, be_u8};

// 解压缩这个地方优化一下
pub fn fast_decrypt(encrypted: &[u8], key: &[u8]) -> Vec<u8> {
    let mut buf = Vec::from(encrypted);
    let mut prev = 0x36;
    for i in 0..buf.len() {
        let mut t = buf[i] >> 4 | buf[i] << 4;
        t = t ^ prev ^ (i as u8) ^ key[i % key.len()];
        prev = buf[i];
        buf[i] = t;
    }
    buf
}


/// nom parser
pub fn text_len_parser_v2(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, len) = be_u16(input)?;
    Ok((input, len + 1))
}

pub fn text_len_parser_v1(input: &[u8]) -> IResult<&[u8], u8> {
    be_u8(input)
}
