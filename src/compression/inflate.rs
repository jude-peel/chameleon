use crate::compression::prefix::{
    Code, PrefixCodeMap, DISTANCE_BASE, DISTANCE_EXTRA_BITS, FIXED_CODE_LENGTHS, LENGTH_BASE,
    LENGTH_EXTRA_BITS,
};

use super::bits::BitVector64;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum DeflateError {
    InvalidBlockError(&'static str),
    InvalidSymbolError(usize, &'static str),
    DecompressionError(&'static str),
}

impl Display for DeflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeflateError::InvalidBlockError(s) => {
                write!(f, "InvalidBlock Error: {}", s)
            }
            DeflateError::InvalidSymbolError(v, r) => {
                write!(f, "InvalidSymbolError caused by symbol: {}, {}", v, r)
            }
            DeflateError::DecompressionError(s) => {
                write!(f, "DecompressionError: {}", s)
            }
        }
    }
}

impl Error for DeflateError {}

#[derive(Debug)]
pub struct DeflateStream {
    pub compressed: Vec<u8>,
    pub decompressed: Vec<u8>,
    pub bitstream: BitVector64,
    pub finished: bool,
}

impl DeflateStream {
    pub fn build(bytes: &[u8]) -> Self {
        let compressed = bytes.to_vec();
        let bitstream = BitVector64::from_be_bytes(bytes);
        let decompressed = Vec::with_capacity(compressed.len());
        let finished = false;

        Self {
            compressed,
            decompressed,
            bitstream,
            finished,
        }
    }
    pub fn decompress(&mut self) -> Result<Vec<u8>, DeflateError> {
        println!("{}", self.bitstream);
        while !self.finished {
            let header = self.bitstream.by_ref().take(3).collect::<Vec<_>>();

            self.finished = matches!(header[0], 1);

            match (header[1], header[2]) {
                (0, 0) => self.store()?,
                (1, 0) => self.fixed()?,
                (0, 1) => self.dynamic()?,
                _ => return Err(DeflateError::InvalidBlockError("Invalid block type.")),
            };
        }

        Ok(self.decompressed.clone())
    }
    fn store(&mut self) -> Result<(), DeflateError> {
        println!("store");
        let len = self
            .bitstream
            .by_ref()
            .skip(5)
            .take(16)
            .fold(0u16, |acc, bit| (acc << 1) | (bit as u16))
            .reverse_bits();

        // Take the subsequent 16 bits as a u16.
        let nlen = self
            .bitstream
            .by_ref()
            .take(16)
            .fold(0u16, |acc, bit| (acc << 1) | (bit as u16))
            .reverse_bits();

        if len != !nlen {
            return Err(DeflateError::InvalidBlockError(
                "Block type is 0, but NLEN is not the bitwise complement to LEN.",
            ));
        }

        // Figure out what byte the current index is in.
        let byte_idx = self.bitstream.idx / 8;

        self.compressed[byte_idx..len as usize + byte_idx]
            .iter()
            .for_each(|x| self.decompressed.push(*x));

        Ok(())
    }
    fn fixed(&mut self) -> Result<(), DeflateError> {
        println!("fixed");

        let code_map = PrefixCodeMap::from_lengths(&FIXED_CODE_LENGTHS);
        let mut code_buf = Code::new();

        let mut output = Vec::new();

        // Iterate through the bitstream.
        while let Some(bit) = self.bitstream.by_ref().next() {
            code_buf.push_bit(bit);
            // Check the code map for the code.
            if let Some(value) = code_map.map.get(&code_buf) {
                code_buf = Code::new();
                // Push literals to the output.
                if *value < 256 {
                    output.push(value);
                } else if let 257..=285 = value {
                    let mut length = LENGTH_BASE[value - 257];
                    let len_extra = LENGTH_EXTRA_BITS[value - 257];

                    // Get the extra length bits.
                    if len_extra > 0 {
                        let additional_length = self
                            .bitstream
                            .by_ref()
                            .take(len_extra as usize)
                            .fold(0u16, |acc, bit| (acc << 1) | bit as u16)
                            .reverse_bits()
                            >> (16 - len_extra);
                        length += additional_length;
                    }

                    // Get the 5 bit distance code.
                    let mut distance = self
                        .bitstream
                        .by_ref()
                        .take(5)
                        .fold(0usize, |acc, bit| (acc << 1) | bit as usize);

                    let dist_extra = DISTANCE_EXTRA_BITS[distance];
                    let dist_base = DISTANCE_BASE[distance];

                    if dist_extra > 0 {
                        let additional_distance = self
                            .bitstream
                            .by_ref()
                            .take(dist_extra as usize)
                            .fold(0u16, |acc, bit| (acc << 1) | bit as u16)
                            .reverse_bits()
                            >> (16 - dist_extra);
                        distance = (dist_base + additional_distance) as usize;
                    } else {
                        distance = dist_base as usize;
                    }

                    let start_idx = output.len() - distance;
                    let end_idx = start_idx + length as usize;

                    for idx in start_idx..end_idx {
                        output.push(output[idx]);
                    }
                } else if *value == 256 {
                    break;
                }
            }
        }
        output
            .iter()
            .map(|x| **x as u8)
            .for_each(|byte| self.decompressed.push(byte));

        Ok(())
    }
    fn dynamic(&mut self) -> Result<(), DeflateError> {
        println!("dynamic");
        // # of literal/length codes - 257 (257..286)
        let hlit = self
            .bitstream
            .by_ref()
            .take(5)
            .fold(0u16, |acc, bit| (acc << 1) | bit as u16)
            .reverse_bits()
            >> (16 - 5);

        // # of distance codes - 1 (1..32)
        let hdist = self
            .bitstream
            .by_ref()
            .take(5)
            .fold(0u8, |acc, bit| (acc << 1) | bit)
            .reverse_bits()
            >> (8 - 5);

        // # of code length codes - 4 (4..19)
        let hclen = self
            .bitstream
            .by_ref()
            .take(4)
            .fold(0u8, |acc, bit| (acc << 1) | bit)
            .reverse_bits()
            >> (8 - 4);

        // Code lengths for the code lengths.
        let cl_len_vec = self
            .bitstream
            .by_ref()
            .take(((hclen + 4) * 3) as usize)
            .collect::<Vec<_>>();

        //
        let mut cl_lengths = [0; 19];
        let mut cl_lengths_sorted = [0; 19];

        // Put code lengths into cl_lengths in the order:
        // 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
        for (i, len) in cl_len_vec.chunks(3).enumerate() {
            let value = len.iter().rev().fold(0u8, |acc, bit| (acc << 1) | *bit);

            cl_lengths[i] = value;
        }
        for (i, len) in cl_lengths_sorted.iter_mut().enumerate() {
            *len = match i {
                16 => cl_lengths[0],
                17 => cl_lengths[1],
                18 => cl_lengths[2],
                0 => cl_lengths[3],
                8 => cl_lengths[4],
                7 => cl_lengths[5],
                9 => cl_lengths[6],
                6 => cl_lengths[7],
                10 => cl_lengths[8],
                5 => cl_lengths[9],
                11 => cl_lengths[10],
                4 => cl_lengths[11],
                12 => cl_lengths[12],
                3 => cl_lengths[13],
                13 => cl_lengths[14],
                2 => cl_lengths[15],
                14 => cl_lengths[16],
                1 => cl_lengths[17],
                15 => cl_lengths[18],
                _ => 0,
            }
        }

        let code_length_map = PrefixCodeMap::from_lengths(&cl_lengths_sorted);

        let mut code_lengths = Vec::new();
        let mut code_buf = Code::new();

        while code_lengths.len() < (hlit as usize + 257 + hdist as usize + 1) {
            if let Some(bit) = self.bitstream.by_ref().next() {
                code_buf.push_bit(bit);
                if let Some(symbol) = code_length_map.map.get(&code_buf) {
                    code_buf = Code::new();
                    match symbol {
                        0..16 => code_lengths.push(*symbol as u8),
                        16..=18 => {
                            let (number_of_extra, base) = match symbol {
                                16 => (2, 3usize),
                                17 => (3, 3usize),
                                _ => (7, 11usize),
                            };
                            let _extra_bits: usize = (self
                                .bitstream
                                .by_ref()
                                .take(number_of_extra)
                                .fold(0u8, |acc, bit| (acc << 1) | bit)
                                .reverse_bits()
                                >> (8 - number_of_extra))
                                as usize;

                            if *symbol == 16 {
                                for _ in 0..(base + _extra_bits) {
                                    code_lengths.push(*code_lengths.last().unwrap());
                                }
                            } else {
                                code_lengths.resize(code_lengths.len() + base + _extra_bits, 0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let ll = PrefixCodeMap::from_lengths(&code_lengths[0..(hlit as usize + 257)]);
        let dist = PrefixCodeMap::from_lengths(&code_lengths[(hlit as usize + 257)..]);

        let mut output = Vec::new();

        let mut ll_buf = Code::new();
        while let Some(bit) = self.bitstream.by_ref().next() {
            ll_buf.push_bit(bit);
            if let Some(symbol) = ll.map.get(&ll_buf) {
                ll_buf = Code::new();
                if *symbol < 256 {
                    output.push(*symbol);
                } else if let 257..285 = symbol {
                    let mut length = LENGTH_BASE[symbol - 257];
                    let len_extra = LENGTH_EXTRA_BITS[symbol - 257];

                    if len_extra > 0 {
                        let additional_length = self
                            .bitstream
                            .by_ref()
                            .take(len_extra as usize)
                            .fold(0u16, |acc, bit| (acc << 1) | bit as u16)
                            .reverse_bits()
                            >> (16 - len_extra);
                        length += additional_length;
                    }

                    let mut _distance = 0usize;
                    loop {
                        let mut dist_buf = Code::new();
                        if let Some(bit) = self.bitstream.by_ref().next() {
                            dist_buf.push_bit(bit);
                            if let Some(dist) = dist.map.get(&dist_buf) {
                                _distance = *dist;
                                break;
                            }
                        }
                    }

                    let dist_extra = DISTANCE_EXTRA_BITS[_distance];
                    let dist_base = DISTANCE_BASE[_distance];

                    if dist_extra > 0 {
                        let additional_distance = self
                            .bitstream
                            .by_ref()
                            .take(dist_extra as usize)
                            .fold(0u16, |acc, bit| (acc << 1) | bit as u16)
                            .reverse_bits()
                            >> (16 - dist_extra);
                        _distance = (dist_base + additional_distance) as usize;
                    } else {
                        _distance = dist_base as usize;
                    }

                    let start_idx = output.len() - _distance;
                    let end_idx = start_idx + length as usize;

                    for idx in start_idx..end_idx {
                        output.push(output[idx]);
                    }
                } else if *symbol == 256 {
                    break;
                }
            }
        }

        output
            .iter()
            .map(|x| *x as u8)
            .for_each(|byte| self.decompressed.push(byte));
        Ok(())
    }
}
