use std::{error::Error, fmt::Display};

use super::{bits::BitVector64, inflate::DeflateStream};

#[derive(Debug)]
pub enum ZlibError {
    InvalidHeader(&'static str),
}

impl Display for ZlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZlibError::InvalidHeader(s) => {
                write!(f, "Error: Invalid header, {}", s)
            }
        }
    }
}

impl Error for ZlibError {}

#[derive(Debug)]
pub struct ZlibHeader {
    pub cm: u8,
    pub cinfo: u8,
    pub fdict: Option<u32>,
    pub flevel: u8,
    pub end_idx: usize,
}

impl ZlibHeader {
    pub fn build(bytes: &[u8]) -> Result<Self, ZlibError> {
        // Took in the order the bytes appear when hexdumped.
        let mut header_stream = BitVector64::from_le_bytes(&bytes[0..2]);

        let mut end_idx = 2;

        let cinfo = header_stream
            .by_ref()
            .take(4)
            .fold(0u8, |acc, bit| (acc << 1) | bit);
        let cm = header_stream
            .by_ref()
            .take(4)
            .fold(0u8, |acc, bit| (acc << 1) | bit);

        let flevel = header_stream
            .by_ref()
            .take(2)
            .fold(0u8, |acc, bit| (acc << 1) | bit);

        let fdict_bool = header_stream
            .by_ref()
            .take(1)
            .fold(0u8, |acc, bit| (acc << 1) | bit)
            == 1;

        let fcheck = u16::from_be_bytes([bytes[0], bytes[1]]);

        if fcheck % 31 != 0 {
            return Err(ZlibError::InvalidHeader(
                "taking the first two bytes as a u16 does not result in a value divisible by 31.",
            ));
        }

        let fdict = if fdict_bool {
            end_idx += 4;
            Some(u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]))
        } else {
            None
        };

        Ok(Self {
            cm,
            cinfo,
            fdict,
            flevel,
            end_idx,
        })
    }
}

#[derive(Debug)]
pub struct ZlibStream {
    pub header: ZlibHeader,
    pub deflate: DeflateStream,
    pub adler32: u32,
}

impl ZlibStream {
    pub fn build(bytes: &[u8]) -> Result<Self, ZlibError> {
        let header = ZlibHeader::build(bytes)?;

        let deflate = DeflateStream::build(&bytes[header.end_idx..bytes.len() - 4]);

        let adler32 = u32::from_be_bytes([
            bytes[bytes.len() - 4],
            bytes[bytes.len() - 3],
            bytes[bytes.len() - 2],
            bytes[bytes.len() - 1],
        ]);

        Ok(Self {
            header,
            deflate,
            adler32,
        })
    }
}
