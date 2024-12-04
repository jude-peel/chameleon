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
        let mut header: [u8; 3] = [0; 3];

        for header_bit in header.iter_mut() {
            if let Some(bit) = self.bitstream.next() {
                *header_bit = bit;
            } else {
                return Err(DeflateError::InvalidBlockError(
                    "Block ran out of bits before a header was specified.",
                ));
            }
        }

        todo!()
    }
}
