use std::{error::Error, fmt::Display};

use crate::{
    compression::bits::BitVector64,
    compression::prefix::{
        PrefixTree, DISTANCE_BASE, DISTANCE_EXTRA_BITS, FIXED_CODE_LENGTHS, LENGTH_BASE,
        LENGTH_EXTRA_BITS,
    },
};

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
                write!(f, "InvalidSymbolError cause by symbol: {}, {}", v, r)
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
    compressed: Vec<u8>,
    decompressed: Vec<u8>,
    pub bitstream: BitVector64,
    finished: bool,
}

impl DeflateStream {
    pub fn build(compressed: &[u8]) -> Self {
        let bitstream = BitVector64::from_be_bytes(compressed);
        Self {
            compressed: compressed.to_vec(),
            decompressed: Vec::new(),
            bitstream,
            finished: false,
        }
    }
    pub fn decompress(&mut self) -> Result<Vec<u8>, DeflateError> {
        while !self.finished {
            // Initialize header.
            let mut header: [u8; 3] = [0; 3];

            // Iterate through header, popping the first 3 items from the
            // bitstream and adding them to header.
            for header_bit in header.iter_mut() {
                if let Some(b) = self.bitstream.next() {
                    *header_bit = b;
                } else {
                    return Err(DeflateError::InvalidBlockError(
                        "Block ran out of bits before a header was specified.",
                    ));
                }
            }

            self.finished = matches!(header[0], 1);

            // Main decompression loop.
            match (header[1], header[2]) {
                (0, 0) => {
                    self.block_type_0()?;
                }
                (1, 0) => {
                    self.block_type_1()?;
                }
                (0, 1) => {
                    self.block_type_2()?;
                }
                _ => return Err(DeflateError::InvalidBlockError("Invalid BTYPE.")),
            }
        }
        Ok(self.decompressed.clone())
    }
    fn block_type_0(&mut self) -> Result<(), DeflateError> {
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
                "BTYPE is 0, but NLEN is not the bitwise complement to LEN.",
            ));
        }

        // Figure out what byte the current index is in.
        let byte_idx = self.bitstream.idx / 8;

        self.compressed[byte_idx..len as usize + byte_idx]
            .iter()
            .for_each(|x| self.decompressed.push(*x));

        Ok(())
    }
    fn block_type_1(&mut self) -> Result<(), DeflateError> {
        let mut prefix_tree = PrefixTree::from_lengths(&FIXED_CODE_LENGTHS);

        //let mut output = Vec::new();

        // Iterate through the bitstream.
        while let Some(bit) = self.bitstream.by_ref().next() {
            // Walk the tree and if there is a value, take it and continue.
            if let Some(value) = prefix_tree.walk(bit) {
                // If the value less than 256, it is a literal and should be
                // pushed unaltered to the output stream.
                if value < 256 {
                    self.decompressed.push(value as u8);
                // If it is in the range from 257..285 it is a length code.
                } else if let 257..=285 = value {
                    // Get the base and number of extra bits.
                    let mut length = LENGTH_BASE[value - 257];
                    let len_extra = LENGTH_EXTRA_BITS[value - 257];
                    // If length has extra bits, iterate through them, and add
                    // the value to the base length.
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

                    // After every length code is a 5 bit distance code.
                    let mut distance: usize = self
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

                    let start_idx = self.decompressed.len() - distance;
                    let end_idx = start_idx + length as usize;

                    for idx in start_idx..end_idx {
                        self.decompressed.push(self.decompressed[idx]);
                    }
                } else if value == 256 {
                    break;
                }
            }
        }

        Ok(())
    }
    fn block_type_2(&mut self) -> Result<(), DeflateError> {
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
        let mut cl_lengths_sorted = [0; 19];

        const LENGTH_ORDER: [usize; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ];

        // Put code lengths into cl_lengths in the order:
        // 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
        for (i, len) in cl_len_vec.chunks(3).enumerate() {
            let value = len.iter().rev().fold(0u8, |acc, bit| (acc << 1) | *bit);

            cl_lengths_sorted[LENGTH_ORDER[i]] = value;
        }

        // Generate the code length prefix tree.
        let mut code_length_tree = PrefixTree::from_lengths(&cl_lengths_sorted);

        let mut code_lengths: Vec<u8> = Vec::new();

        while code_lengths.len() < (hlit as usize + 257 + hdist as usize + 1) {
            if let Some(bit) = self.bitstream.by_ref().next() {
                if let Some(symbol) = code_length_tree.walk(bit) {
                    match symbol {
                        0..16 => code_lengths.push(symbol as u8),
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

                            if symbol == 16 {
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

        let mut ll_tree = PrefixTree::from_lengths(&code_lengths[0..(hlit as usize + 257)]);
        let mut dist_tree = PrefixTree::from_lengths(&code_lengths[(hlit as usize + 257)..]);

        // Nearly identical logic to block type 1.
        while let Some(bit) = self.bitstream.by_ref().next() {
            if let Some(sym) = ll_tree.walk(bit) {
                if sym < 256 {
                    self.decompressed.push(sym as u8);
                } else if let 257..285 = sym {
                    let mut length = LENGTH_BASE[sym - 257];
                    let len_extra = LENGTH_EXTRA_BITS[sym - 257];

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

                    // Distance codes are encoded.
                    let mut distance: usize;
                    loop {
                        if let Some(bit) = self.bitstream.by_ref().next() {
                            if let Some(dist) = dist_tree.walk(bit) {
                                distance = dist;
                                break;
                            }
                        }
                    }

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

                    let start_idx = self.decompressed.len().saturating_sub(distance);
                    let end_idx = start_idx + length as usize;

                    for idx in start_idx..end_idx {
                        self.decompressed.push(self.decompressed[idx]);
                    }
                } else if sym == 256 {
                    break;
                }
            }
        }

        Ok(())
    }
}
