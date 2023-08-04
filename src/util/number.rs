use std::fs::File;
use std::io::{BufReader, Read};

use byteorder::{BigEndian, ReadBytesExt};

pub struct NumberBytes {
    tail: Vec<u8>,
    number_width: u8,
}

impl NumberBytes {
    pub fn new(bytes: &Vec<u8>, number_width: u8) -> Self {
        NumberBytes {
            tail: bytes.clone(),
            number_width,
        }
    }

    pub fn read_number(&mut self) -> Option<u64> {
        let cur_tail = self.tail.clone();
        if cur_tail.len() < self.number_width as usize {
            return None;
        }
        let (mut num, tail_bytes) = cur_tail.split_at(self.number_width as usize);
        self.tail = Vec::from(tail_bytes);
        Some(num.read_u64::<BigEndian>().unwrap())
    }
}

pub fn read_number_from_be_bytes(reader: &mut BufReader<File>, width: u8) -> usize {
    let mut buf = vec![0; width as usize];
    reader
        .read_exact(&mut buf)
        .expect("buffer read exact error");
    match width {
        8 => u64::from_be_bytes(buf.try_into().unwrap()) as usize,
        4 => u32::from_be_bytes(buf.try_into().unwrap()) as usize,
        2 => u16::from_be_bytes(buf.try_into().unwrap()) as usize,
        _ => 0,
    }
}
