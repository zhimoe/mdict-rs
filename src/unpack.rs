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
pub trait ReadBytes: Sized {
    type Number;
    fn read_number(slice: &[u8], byteorder: Endian) -> anyhow::Result<Self::Number>;
}

impl ReadBytes for u16 {
    type Number = u16;
    fn read_number(slice: &[u8], byteorder: Endian) -> anyhow::Result<u16> {
        let mut out: &[u8] = slice.clone();
        let num = match byteorder {
            Endian::LE => out.read_u16::<LittleEndian>()?,
            Endian::BE => out.read_u16::<BigEndian>()?,
        };
        Ok(num)
    }
}

impl ReadBytes for u32 {
    type Number = u32;
    fn read_number(slice: &[u8], byteorder: Endian) -> anyhow::Result<u32> {
        let mut out: &[u8] = slice.clone();
        let num = match byteorder {
            Endian::LE => out.read_u32::<LittleEndian>()?,
            Endian::BE => out.read_u32::<BigEndian>()?,
        };
        Ok(num)
    }
}

impl ReadBytes for u64 {
    type Number = u64;
    fn read_number(slice: &[u8], byteorder: Endian) -> anyhow::Result<u64> {
        let mut out: &[u8] = slice.clone();
        let num = match byteorder {
            Endian::LE => out.read_u64::<LittleEndian>()?,
            Endian::BE => out.read_u64::<BigEndian>()?,
        };
        Ok(num)
    }
}

pub fn unpack<R: ReadBytes>(slice: &[u8], byteorder: Endian) -> anyhow::Result<R::Number> {
    R::read_number(slice, byteorder)
}

#[test]
fn test() {
    let slice = [0, 0, 2, 206];
    let n = unpack::<u32>(&slice, Endian::BE);
    assert_eq!(n.unwrap(), 718);
}

/// utf16 little endian bytes to string, equals python bytes.decode("utf-16-le")
pub fn string_from_utf16_le(slice: &[u8]) -> anyhow::Result<String> {
    // step 1: convert [u8] to [u16] in little endian, slice len must be even
    let packets = slice
        .chunks(2)
        .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
        .collect::<Vec<_>>();
    // step 2: convert to string
    let s = String::from_utf16(&packets)?;
    Ok(s)
}

#[test]
fn utf16_string_le_test() {}
