use std::io::prelude::*;

use flate2::read::ZlibDecoder;

/// decompress compressed bytes
pub fn decompress(bytes: &[u8], out_buf: &mut Vec<u8>) {
    let mut decoder = ZlibDecoder::new(bytes);
    decoder
        .read_to_end(out_buf)
        .expect("decompress read_to_end() error");
}

#[test]
fn test() -> anyhow::Result<()> {
    // prepare
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    let text = "Hello World!";
    // encode_buf只是一个缓存区，不是压缩后的bytes存放变量，实际上encode_buf所有权会move
    // 压缩后的结果通过finish方法返回
    let encode_buf: Vec<u8> = vec![];
    let mut encoder = ZlibEncoder::new(encode_buf, Compression::default());
    encoder.write_all(text.as_bytes()).unwrap();
    let compressed_bytes = encoder.finish()?;
    println!("compressed_bytes = {:?}", &compressed_bytes);

    // start test
    let mut decompressed = Vec::<u8>::new();
    decompress(&compressed_bytes, &mut decompressed);

    assert_eq!("Hello World!", String::from_utf8_lossy(&decompressed));
    Ok(())
}
