use std::fs::File;
use std::io::{BufReader, Read};

/// numbers from big endian bytes and use as an iterator
pub struct NumberFromBeBytes {
    numbers: Vec<u64>,
    index: usize,
}

impl Iterator for NumberFromBeBytes {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.numbers.len() {
            None
        } else {
            self.index += 1;
            Some(self.numbers[self.index - 1])
        }
    }
}

impl NumberFromBeBytes {
    pub fn new(bytes: &[u8], width: u8) -> Self {
        let width = width as usize;
        let mut numbers = Vec::new();
        let num_count = bytes.len() / width;

        for i in 0..num_count {
            let start = i * width;
            let end = start + width;
            let slice = &bytes[start..end];

            let num = match width {
                8 => u64::from_be_bytes(slice.try_into().unwrap()),
                4 => u32::from_be_bytes(slice.try_into().unwrap()) as u64,
                2 => u16::from_be_bytes(slice.try_into().unwrap()) as u64,
                _ => panic!("Invalid width!"),
            };

            numbers.push(num);
        }
        NumberFromBeBytes { numbers, index: 0 }
    }
}

/// read bytes from a buffer reader and convert it into number with big endian
pub fn read_number_from_be_bytes(reader: &mut BufReader<File>, width: u8) -> usize {
    let mut buf = vec![0; width as usize];
    reader
        .read_exact(&mut buf)
        .expect("buffer read_exact() error");
    match width {
        8 => u64::from_be_bytes(buf.try_into().unwrap()) as usize,
        4 => u32::from_be_bytes(buf.try_into().unwrap()) as usize,
        2 => u16::from_be_bytes(buf.try_into().unwrap()) as usize,
        _ => 0,
    }
}
