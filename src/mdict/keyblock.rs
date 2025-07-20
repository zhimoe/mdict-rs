use crate::mdict::header::{Header, Version};
use crate::util::fast_decrypt;
use crate::util::text_len_parser_v1;
use crate::util::text_len_parser_v2;
use adler32::adler32;
use encoding::label::encoding_from_whatwg_label;
use flate2::read::ZlibDecoder;
use nom::{
    IResult, Parser,
    bytes::complete::{take, take_till},
    combinator::map,
    multi::{length_data, many0},
    number::complete::{be_u32, be_u64, le_u32},
};
use ripemd::{Digest, Ripemd128};
use std::{io::Read, str};

pub struct KeyBlockHeader {
    #[allow(unused)]
    pub block_num: usize,
    #[allow(unused)]
    pub entry_num: usize,
    // only version >= 2
    #[allow(unused)]
    pub key_block_info_decompressed_len: usize,
    pub key_block_info_len: usize,
    pub key_blocks_len: usize,
}

/// every key block compressed size and decompressed size
/// 用于解析出 RecordEntry list
pub struct KeyBlockSize {
    pub csize: usize,
    pub dsize: usize,
}

/// 词典索引信息, 和实体词典的索引一样，一个text以及一个页码，不过这个页码是整个RecordBlock解压后(叫debuf)的偏移量
#[derive(Debug)]
pub struct RecordDeBufOffset {
    pub text: String,
    // record在所有RecordBlock解压后的起始位置
    pub record_offset_in_debuf: usize,
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
            (be_u32, be_u32, be_u32, be_u32),
            |(block_num, entry_num, info_len, blocks_len)| KeyBlockHeader {
                block_num: block_num as usize,
                entry_num: entry_num as usize,
                key_block_info_decompressed_len: info_len as usize, // 没有压缩则相等
                key_block_info_len: info_len as usize,
                key_blocks_len: blocks_len as usize,
            },
        )
        .parse(info_buf)?;
        Ok((data, kbh))
    }

    fn parse_key_block_header_v2(data: &[u8]) -> IResult<&[u8], KeyBlockHeader> {
        // 5个元信息 和 v1相比多了一个key_block_info_decompressed_size 和一个 adler32 checksum
        let (data, info_buf) = take(40_usize)(data)?;
        let (data, checksum) = be_u32(data)?;

        // checksum info_buf
        assert_eq!(adler32(info_buf).unwrap(), checksum);
        let (_, kbh) = map(
            (be_u64, be_u64, be_u64, be_u64, be_u64),
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
        )
        .parse(info_buf)?;
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

    fn v1(data: &[u8], block_info_len: usize) -> IResult<&[u8], Vec<KeyBlockSize>> {
        let (data, block_info) = take(block_info_len)(data)?;
        let key_blocks_size = decode_key_blocks_size_v1(block_info);
        Ok((data, key_blocks_size))
    }

    fn v2<'a>(
        data: &'a [u8],
        block_info_len: usize,
        encrypted: &str,
    ) -> IResult<&'a [u8], Vec<KeyBlockSize>> {
        let (data, block_info) = take(block_info_len)(data)?;
        assert_eq!(&block_info[0..4], b"\x02\x00\x00\x00");

        let mut key_block_info = vec![];

        if encrypted == "0" {
            ZlibDecoder::new(&block_info[8..])
                .read_to_end(&mut key_block_info)
                .unwrap();
        }

        //decrypt
        if encrypted == "2" || encrypted == "3" {
            let mut md = Ripemd128::new();
            let mut v = Vec::from(&block_info[4..8]);
            let value: u32 = 0x3695;
            v.extend_from_slice(&value.to_le_bytes());
            md.update(v);
            let key = md.finalize();
            let mut d = Vec::from(&block_info[0..8]);
            let decrypted = fast_decrypt(&block_info[8..], key.as_slice());
            d.extend(decrypted);
            ZlibDecoder::new(&d[8..])
                .read_to_end(&mut key_block_info)
                .unwrap();
        }

        let key_blocks_size = decode_key_blocks_size_v2(&key_block_info[..]);
        Ok((data, key_blocks_size))
    }

    /// number of entries, num of bytes, first, num of bytes, last?
    fn decode_key_blocks_size_v1(block_info: &[u8]) -> Vec<KeyBlockSize> {
        let mut parser = many0(map(
            (
                be_u32,
                length_data(text_len_parser_v1),
                length_data(text_len_parser_v1),
                be_u32,
                be_u32,
            ),
            |(_, _, _, csize, dsize)| KeyBlockSize {
                csize: csize as usize,
                dsize: dsize as usize,
            },
        ));
        let (remain, res) = parser.parse(block_info).unwrap();
        assert_eq!(
            remain.len(),
            0,
            "failed: key block info parser left some data"
        );
        res
    }

    fn decode_key_blocks_size_v2(block_info: &[u8]) -> Vec<KeyBlockSize> {
        let mut parser = many0(map(
            (
                be_u64,
                length_data(text_len_parser_v2),
                length_data(text_len_parser_v2),
                be_u64,
                be_u64,
            ),
            |(_, _, _, csize, dsize)| KeyBlockSize {
                csize: csize as usize,
                dsize: dsize as usize,
            },
        ));
        let (remain, res) = parser.parse(block_info).unwrap();
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
) -> IResult<&'a [u8], Vec<RecordDeBufOffset>> {
    let (data, buf) = take(key_blocks_len)(data)?;
    let mut buf = buf;

    let mut key_entries: Vec<RecordDeBufOffset> = vec![];

    for block_size in key_blocks_size.iter() {
        let (remain, decompressed) =
            key_block_parser(block_size.csize, block_size.dsize).parse(buf)?;
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
fn parse_block_items_v1<'a>(
    data: &'a [u8],
    encoding: &'a str,
) -> IResult<&'a [u8], Vec<RecordDeBufOffset>> {
    let (remain, entries) = many0(map(
        (be_u32, take_till(|x| x == 0), take(1_usize)),
        |(offset, buf, _)| {
            let decoder = encoding_from_whatwg_label(encoding).unwrap();
            let text = decoder.decode(buf, encoding::DecoderTrap::Ignore).unwrap();
            RecordDeBufOffset {
                record_offset_in_debuf: offset as usize,
                text,
            }
        },
    ))
    .parse(data)?;

    assert_eq!(remain.len(), 0);

    Ok((remain, entries))
}

fn parse_block_items_v2<'a>(
    data: &'a [u8],
    encoding: &'a str,
) -> IResult<&'a [u8], Vec<RecordDeBufOffset>> {
    let (remain, sep) = many0(map(
        (be_u64, take_till(|x| x == 0), take(1_usize)),
        |(offset, buf, _end_zero)| {
            let decoder = encoding_from_whatwg_label(encoding).unwrap();
            let text = decoder.decode(buf, encoding::DecoderTrap::Ignore).unwrap();
            RecordDeBufOffset {
                record_offset_in_debuf: offset as usize,
                text,
            }
        },
    ))
    .parse(data)?;

    assert_eq!(remain.len(), 0);

    Ok((remain, sep))
}

/// 解析一个 key block 得到的是bytes
fn key_block_parser<'a>(
    csize: usize,
    dsize: usize,
) -> impl Parser<&'a [u8], Output = Vec<u8>, Error = nom::error::Error<&'a [u8]>> {
    map(
        (le_u32, take(4_usize), take(csize - 8)),
        move |(enc, checksum, encrypted_buf)| {
            let enc_method = (enc >> 4) & 0xf;
            let comp_method = enc & 0xf;

            let mut md = Ripemd128::new();
            md.update(checksum);
            let key = md.finalize();
            // todo: 这一段好像和结构图中不太一样
            let data: Vec<u8> = match enc_method {
                0 => Vec::from(encrypted_buf),
                1 => fast_decrypt(encrypted_buf, key.as_slice()),
                2 => {
                    let decrypt = vec![];
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
                    ZlibDecoder::new(&data[..]).read_to_end(&mut v).unwrap();
                    v
                }
                _ => panic!("unknown compression method: {}", comp_method),
            };

            decompressed
        },
    )
}
