use std::{io::Read, str};

use adler32::adler32;
use compress::zlib;
use encoding::{Encoding, label::encoding_from_whatwg_label};
use nom::{
    bytes::complete::{take, take_till},
    combinator::map,
    IResult,
    multi::{length_data, many0},
    number::complete::{be_u32, be_u64, le_u32},
    sequence::tuple, Slice,
};
use ripemd::{Digest, Ripemd128};
use salsa20::{cipher::KeyIvInit, Salsa20};

use crate::mdict::header::{Header, Version};
use crate::util::fast_decrypt;
use crate::util::text_len_parser_v1;
use crate::util::text_len_parser_v2;

#[derive(Debug)]
pub struct KeyBlockHeader {
    pub block_num: usize,
    pub entry_num: usize,
    // only version >= 2
    pub key_block_info_decompressed_len: usize,
    pub key_block_info_len: usize,
    pub key_blocks_len: usize,
}

/// every key block compressed size and decompressed size
/// 用于解析出 RecordEntry list
#[derive(Debug)]
pub struct KeyBlockSize {
    pub csize: usize,
    pub dsize: usize,
}

/// 词典索引信息, 和实体词典的索引一样，一个text以及一个页码，不过这个页码是整个buf解压后的偏移量
#[derive(Debug)]
pub struct Entry {
    pub text: String,
    // 整个buf解压缩后entry的偏移量
    pub buf_decompressed_offset: usize,
}

pub fn parse_key_block_header<'a>(
    data: &'a [u8],
    header: &'a Header,
) -> IResult<&'a [u8], KeyBlockHeader> {
    return match header.version {
        Version::V1 => parse_key_block_header_v1(data),
        Version::V2 => parse_key_block_header_v2(data),
    };

    fn parse_key_block_header_v1(data: &[u8]) -> IResult<&[u8], KeyBlockHeader> {
        let (data, info_buf) = take(16_usize)(data)?;
        // map 接收一个parser和一个匿名fn, 将parser的结果传递给fn后得到返回值
        let (_, kbh) = map(
            tuple((be_u32, be_u32, be_u32, be_u32)),
            |(block_num, entry_num, info_len, blocks_len)| KeyBlockHeader {
                block_num: block_num as usize,
                entry_num: entry_num as usize,
                key_block_info_decompressed_len: info_len as usize, // 没有压缩则相等
                key_block_info_len: info_len as usize,
                key_blocks_len: blocks_len as usize,
            },
        )(info_buf)?;
        Ok((data, kbh))
    }

    fn parse_key_block_header_v2(data: &[u8]) -> IResult<&[u8], KeyBlockHeader> {
        // 5个元信息 和 v1相比多了一个key_block_info_decompressed_size 和一个 adler32 checksum
        let (data, info_buf) = take(40_usize)(data)?;
        let (data, checksum) = be_u32(data)?;

        // checksum info_buf
        assert_eq!(adler32(info_buf).unwrap(), checksum);

        let (_, kbh) = map(
            tuple((be_u64, be_u64, be_u64, be_u64, be_u64)),
            |(
                 block_num,
                 entry_num,
                 key_block_info_decompressed_len,
                 key_block_info_len,
                 key_blocks_len,
             )| KeyBlockHeader {
                block_num: block_num as usize,
                entry_num: entry_num as usize,
                key_block_info_decompressed_len: key_block_info_decompressed_len as usize,
                key_block_info_len: key_block_info_len as usize,
                key_blocks_len: key_blocks_len as usize,
            },
        )(info_buf)?;
        Ok((data, kbh))
    }
}

/// Vec<(usize,usize)>: every key block compressed and decompressed size
pub fn parse_key_block_info<'a>(
    data: &'a [u8],
    block_info_len: usize,
    header: &'a Header,
) -> IResult<&'a [u8], Vec<KeyBlockSize>> {
    return match &header.version {
        Version::V1 => v1(data, block_info_len),
        Version::V2 => v2(data, block_info_len, &header.encrypted),
    };

    fn v1<'a>(data: &'a [u8], block_info_len: usize) -> IResult<&'a [u8], Vec<KeyBlockSize>> {
        let (data, block_info) = take(block_info_len)(data)?;
        let key_blocks_size = decode_key_blocks_size_v1(block_info);
        Ok((data, key_blocks_size))
    }

    fn v2<'a>(
        data: &'a [u8],
        block_info_len: usize,
        encrypted: &str,
    ) -> IResult<&'a [u8], Vec<KeyBlockSize>> {
        let (left, block_info) = take(block_info_len)(data)?;
        assert_eq!(block_info.slice(0..4), b"\x02\x00\x00\x00");

        let mut key_block_info = vec![];

        if encrypted == "0" {
            zlib::Decoder::new(&block_info[8..])
                .read_to_end(&mut key_block_info)
                .unwrap();
        }

        //decrypt
        if encrypted == "2" || encrypted == "3" {
            let mut md = Ripemd128::new();
            let mut v = Vec::from(block_info.slice(4..8));
            let value: u32 = 0x3695;
            v.extend_from_slice(&value.to_le_bytes());
            md.update(v);
            let key = md.finalize();
            let mut d = Vec::from(&block_info[0..8]);
            let decrypted = fast_decrypt(&block_info[8..], key.as_slice());
            d.extend(decrypted);
            zlib::Decoder::new(&d[8..])
                .read_to_end(&mut key_block_info)
                .unwrap();
        }

        let entry_infos = decode_key_blocks_size_v2(&key_block_info[..]);
        Ok((left, entry_infos))
    }

    /// number of entries, num of bytes, first, num of bytes, last?
    fn decode_key_blocks_size_v1(block_info: &[u8]) -> Vec<KeyBlockSize> {
        let mut parser = many0(map(
            tuple((
                be_u32,
                length_data(text_len_parser_v1),
                length_data(text_len_parser_v1),
                be_u32,
                be_u32,
            )),
            |(_, _, _, csize, dsize)| KeyBlockSize {
                csize: csize as usize,
                dsize: dsize as usize,
            },
        ));
        let (remain, res) = parser(block_info).unwrap();
        assert_eq!(
            remain.len(),
            0,
            "failed: key block info parser left some data"
        );
        res
    }

    fn decode_key_blocks_size_v2(block_info: &[u8]) -> Vec<KeyBlockSize> {
        let mut parser = many0(map(
            tuple((
                be_u64,
                length_data(text_len_parser_v2),
                length_data(text_len_parser_v2),
                be_u64,
                be_u64,
            )),
            |(_, _, _, csize, dsize)| KeyBlockSize {
                csize: csize as usize,
                dsize: dsize as usize,
            },
        ));
        let (remain, res) = parser(block_info).unwrap();
        assert_eq!(remain.len(), 0);
        res
    }
}

/// 解析 key blocks
pub fn parse_key_blocks<'a>(
    data: &'a [u8],
    key_blocks_len: usize,
    header: &Header,
    key_blocks_size: &'a Vec<KeyBlockSize>,
) -> IResult<&'a [u8], Vec<Entry>> {
    let (data, buf) = take(key_blocks_len)(data)?;
    let mut buf = buf;

    let mut key_entries: Vec<Entry> = vec![];

    for info in key_blocks_size.iter() {
        let (remain, decompressed) = key_block_parser(info.csize, info.dsize)(buf)?;
        let (_, mut one_block_entries) = match &header.version {
            Version::V1 => parse_block_items_v1(&decompressed[..], &header.encoding).unwrap(),
            Version::V2 => parse_block_items_v2(&decompressed[..], &header.encoding).unwrap(),
        };

        buf = remain;
        key_entries.append(&mut one_block_entries);
    }

    Ok((data, key_entries))
}

// TODO 可以合并
fn parse_block_items_v1<'a>(data: &'a [u8], encoding: &'a str) -> IResult<&'a [u8], Vec<Entry>> {
    let (remain, entries) = many0(map(
        tuple((be_u32, take_till(|x| x == 0), take(1_usize))),
        |(offset, buf, _)| {
            let decoder = encoding_from_whatwg_label(encoding).unwrap();
            let text = decoder.decode(buf, encoding::DecoderTrap::Ignore).unwrap();
            Entry {
                buf_decompressed_offset: offset as usize,
                text,
            }
        },
    ))(data)?;

    assert_eq!(remain.len(), 0);

    Ok((remain, entries))
}

fn parse_block_items_v2<'a>(data: &'a [u8], encoding: &'a str) -> IResult<&'a [u8], Vec<Entry>> {
    let (remain, sep) = many0(map(
        tuple((be_u64, take_till(|x| x == 0), take(1_usize))),
        |(offset, buf, _)| {
            let decoder = encoding_from_whatwg_label(encoding).unwrap();
            let text = decoder.decode(buf, encoding::DecoderTrap::Ignore).unwrap();
            Entry {
                buf_decompressed_offset: offset as usize,
                text,
            }
        },
    ))(data)?;

    assert_eq!(remain.len(), 0);

    Ok((remain, sep))
}

/// 解析一个 key block 得到的是bytes
fn key_block_parser<'a>(
    csize: usize,
    dsize: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
    map(
        tuple((le_u32, take(4_usize), take(csize - 8))),
        move |(enc, checksum, encrypted)| {
            let enc_method = (enc >> 4) & 0xf;
            let _enc_size = (enc >> 8) & 0xff;
            let comp_method = enc & 0xf;

            let mut md = Ripemd128::new();
            md.update(checksum);
            let key = md.finalize();

            let data: Vec<u8> = match enc_method {
                0 => Vec::from(encrypted),
                1 => fast_decrypt(encrypted, key.as_slice()),
                2 => {
                    let decrypt = vec![];
                    let mut cipher = Salsa20::new(key.as_slice().into(), &[0; 8].into());
                    decrypt
                }
                _ => panic!("unknown enc method: {}", enc_method),
            };

            let decompressed = match comp_method {
                0 => data,
                1 => {
                    let mut comp: Vec<u8> = vec![0xf0];
                    comp.extend_from_slice(&data[..]);
                    let lzo = minilzo_rs::LZO::init().unwrap();
                    lzo.decompress(&data[..], dsize).unwrap()
                }
                2 => {
                    let mut v = vec![];
                    zlib::Decoder::new(&data[..]).read_to_end(&mut v).unwrap();
                    v
                }
                _ => panic!("unknown compression method: {}", comp_method),
            };

            decompressed
        },
    )
}
