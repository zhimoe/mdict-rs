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
