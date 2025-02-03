use std::io::Read;

use flate2::read::ZlibDecoder;
use nom::bytes::complete::take;
use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::{be_u32, be_u64, le_u32};
use nom::{IResult, Parser};
use ripemd::{Digest, Ripemd128};

use crate::mdict::header::{Header, Version};
use crate::util::fast_decrypt;

/// every record block compressed size and decompressed size
#[derive(Debug)]
pub struct RecordBlockSize {
    pub csize: usize,
    pub dsize: usize,
}

pub fn parse_record_blocks<'a>(
    data: &'a [u8],
    header: &'a Header,
) -> IResult<&'a [u8], Vec<RecordBlockSize>> {
    match &header.version {
        Version::V1 => parse_record_blocks_v1(data),
        Version::V2 => parse_record_blocks_v2(data),
    }
}

fn parse_record_blocks_v1(data: &[u8]) -> IResult<&[u8], Vec<RecordBlockSize>> {
    let (data, (records_num, _entries_num, record_info_len, _record_buf_len)) =
        (be_u32, be_u32, be_u32, be_u32).parse(data)?;

    assert_eq!(records_num * 8, record_info_len);

    count(
        map((be_u32, be_u32), |(csize, dsize)| RecordBlockSize {
            csize: csize as usize,
            dsize: dsize as usize,
        }),
        records_num as usize,
    )
        .parse(data)
}

fn parse_record_blocks_v2(data: &[u8]) -> IResult<&[u8], Vec<RecordBlockSize>> {
    let (data, (records_num, _entries_num, record_info_len, _record_buf_len)) =
        (be_u64, be_u64, be_u64, be_u64).parse(data)?;

    assert_eq!(records_num * 16, record_info_len,);

    count(
        map((be_u64, be_u64), |(csize, dsize)| RecordBlockSize {
            csize: csize as usize,
            dsize: dsize as usize,
        }),
        records_num as usize,
    )
        .parse(data)
}

pub(crate) fn record_block_parser<'a>(
    size: usize,
    dsize: usize,
) -> impl Parser<&'a [u8], Output=Vec<u8>, Error=nom::error::Error<&'a [u8]>> {
    map(
        (le_u32, take(4_usize), take(size - 8)),
        move |(enc, checksum, encrypted)| {
            // 规范里面好像没有加密这步
            let enc_method = (enc >> 4) & 0xf;
            let comp_method = enc & 0xf;

            let mut md = Ripemd128::new();
            md.update(checksum);
            let key = md.finalize();

            let data: Vec<u8> = match enc_method {
                0 => Vec::from(encrypted),
                1 => fast_decrypt(encrypted, key.as_slice()),
                2 => {
                    let decrypt = vec![];
                    decrypt
                }
                _ => panic!("unknown enc method: {}", enc_method),
            };

            let decompressed = match comp_method {
                0 => data,
                1 => {
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
