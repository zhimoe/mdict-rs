use adler32::RollingAdler32;


fn main() {

    //{0x118e038e, "abcdefghi", "adl\x01\x03\xd8\x01\x8b"},

    let contents = "abcdefghi";
    let mut rolling_adler32 = RollingAdler32::new();
    rolling_adler32.update_buffer(&contents.as_bytes());
    let hash = rolling_adler32.hash();
    println!("{:x}", hash);//0x118e038e
}