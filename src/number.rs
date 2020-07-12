use std::fs::File;
use std::io::{BufReader, Read};

use byteorder::{BigEndian, ReadBytesExt};

pub struct NumberBytes {
    tail: Vec<u8>,
}

impl NumberBytes {
    pub fn new(bytes: &Vec<u8>) -> Self {
        NumberBytes {
            tail: bytes.clone(),
        }
    }

    pub fn read_number(&mut self, width: usize) -> Option<u64> {
        let cur_tail = self.tail.clone();
        if cur_tail.len() < width {
            return None;
        }
        let (mut num, tail_bytes) = cur_tail.split_at(width);
        self.tail = Vec::from(tail_bytes);
        Some(num.read_u64::<BigEndian>().unwrap())
    }
}

pub fn read_number(reader: &mut BufReader<File>, width: usize) -> usize {
    let mut buf: Vec<u8> = vec![0; width];
    reader.read_exact(&mut buf).expect("read_exact error");
    let mut slice = &buf[..];
    return match width {
        8 => slice.read_u64::<BigEndian>().unwrap() as usize,
        4 => slice.read_u32::<BigEndian>().unwrap() as usize,
        2 => slice.read_u16::<BigEndian>().unwrap() as usize,
        _ => 0,
    };
}

