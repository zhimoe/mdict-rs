use adler32::RollingAdler32;

/// adler32 checksum 完整性检测
pub fn adler32_checksum(contents: &[u8], checksum: u32) -> bool {
    let mut rolling_adler32 = RollingAdler32::new();
    rolling_adler32.update_buffer(contents);
    let hash = rolling_adler32.hash();
    return hash & 0xffffffff == checksum;
}

#[test]
fn test() {
    let contents = "abcdefghi";
    let mut rolling_adler32 = RollingAdler32::new();
    rolling_adler32.update_buffer(&contents.as_bytes());
    let hash = rolling_adler32.hash();
    assert_eq!(0x118e038e, hash);
}
