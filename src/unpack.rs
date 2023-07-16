use std::string::FromUtf16Error;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

pub enum Endian {
    // LittleEndian
    LE,
    // BigEndian
    BE,
}

// big endian bytes unpack,等价下面的 python 代码
// from struct import unpack
// unpack('>I', bytes_arr)
// unpack('<I', bytes_arr)
pub fn unpack_u16(bytes: &[u8], byteorder: Endian) -> u16 {
    let mut out: &[u8] = bytes.clone();
    match byteorder {
        Endian::LE => out.read_u16::<LittleEndian>().unwrap(),
        Endian::BE => out.read_u16::<BigEndian>().unwrap(),
    }
}

pub fn unpack_u32(bytes: &[u8], byteorder: Endian) -> u32 {
    let mut out: &[u8] = bytes.clone();
    match byteorder {
        Endian::LE => out.read_u32::<LittleEndian>().unwrap(),
        Endian::BE => out.read_u32::<BigEndian>().unwrap(),
    }
}

pub fn unpack_u64(bytes: &[u8], byteorder: Endian) -> u64 {
    let mut out: &[u8] = bytes.clone();
    match byteorder {
        Endian::LE => out.read_u64::<LittleEndian>().unwrap(),
        Endian::BE => out.read_u64::<BigEndian>().unwrap(),
    }
}

/// utf16 little endian bytes to string, equals python bytes.decode("utf-16-le")
pub fn string_from_utf16_le(slice: &[u8]) -> Result<String, FromUtf16Error> {
    // step 1: convert [u8] to [u16] in little endian, slice len must be even
    let packets = slice
        .chunks(2)
        .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
        .collect::<Vec<_>>();
    // step 2: convert to string
    String::from_utf16(&packets)
}

#[test]
fn utf16_string_le_test() {}
