use std::{
    error::Error,
    fmt::{self, Display},
    fs, io,
    path::Path,
    str,
};

use crate::compression::{crc, zlib::ZlibStream};

pub const PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const VALID_CHUNK_TYPES: [&str; 18] = [
    "IHDR", "PLTE", "IDAT", "IEND", "cHRM", "gAMA", "iCCP", "sBIT", "sRGB", "bKGD", "hIST", "tRNS",
    "pHYs", "sPLT", "tIME", "iTXt", "tEXt", "zTXt",
];

pub struct Png {
    pub data: PngData,
}

impl Png {
    /// Creates a Png struct from the given path.
    ///
    /// # Arguments
    ///
    /// * 'path' - The file path to the PNG file, can be any type that implements into path.
    ///
    /// # Returns
    ///
    /// A result containing either the constructed Png or a DecoderError.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Png, DecoderError> {
        let path = path.as_ref();

        let file_bytes = fs::read(path)?;

        if file_bytes[0..8] != PNG_HEADER {
            return Err(DecoderError::TypeError(format!("{:?} is not a PNG.", path)));
        }

        let data = PngData::build(&file_bytes)?;

        Ok(Png { data })
    }
    pub fn rgb(&self) -> Vec<(u8, u8, u8)> {
        // Concatenate the data from all IDAT chunks.
        let zlib_bytes = self
            .data
            .idat
            .iter()
            .flat_map(|ch| &ch.data)
            .cloned()
            .collect::<Vec<_>>();

        let zlib = ZlibStream::build(&zlib_bytes).unwrap();

        todo!()
    }
}

#[derive(Clone)]
pub struct Chunk {
    length: usize,
    ctype: String,
    data: Vec<u8>,
    crc: u32,
    size: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            length: 0,
            ctype: String::new(),
            data: Vec::new(),
            crc: 0,
            size: 0,
        }
    }
    pub fn from(bytes: &[u8]) -> Result<Self, DecoderError> {
        let mut byte_iterator = bytes.iter();

        let length = byte_iterator
            .by_ref()
            .take(4)
            .fold(0usize, |acc, byte| (acc << 8) | *byte as usize);

        let type_vec = byte_iterator.by_ref().take(4).cloned().collect::<Vec<_>>();
        let ctype = match String::from_utf8(type_vec.clone()) {
            Ok(str) => {
                if VALID_CHUNK_TYPES.contains(&str.as_str()) {
                    str
                } else {
                    return Err(DecoderError::InvalidChunk("chunk type is invalid."));
                }
            }
            Err(_) => {
                return Err(DecoderError::InvalidChunk(
                    "could not convert type to utf-8.",
                ));
            }
        };

        let data = byte_iterator
            .by_ref()
            .take(length)
            .cloned()
            .collect::<Vec<_>>();

        let crc = byte_iterator
            .by_ref()
            .take(4)
            .fold(0u32, |acc, byte| (acc << 8) | *byte as u32);

        let to_hash = [type_vec, data.clone()].concat();

        if crc != crc::hash(&to_hash) {
            return Err(DecoderError::InvalidChunk(
                "chunk CRC could not be verified.",
            ));
        }

        let size = length + 12;

        Ok(Self {
            length,
            ctype,
            data,
            crc,
            size,
        })
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

/// A struct containing the roughly parsed data of a PNG file.
///
/// # Fields
///
/// * 'raw_data' - A Vec<u8> containing the raw byte data.
/// * 'ihdr' - An array storing the 13 byte IHDR chunk.
/// * 'plte' - Contains the optional PLTE chunk.
/// * 'IDAT' - Contains a vector of Vec<u8>'s containing the IDAT chunk/chunks.
pub struct PngData {
    pub raw_data: Vec<u8>,
    pub ihdr: Chunk,
    pub plte: Option<Chunk>,
    pub idat: Vec<Chunk>,
    pub ancillary_chunks: Vec<Chunk>,
}

impl PngData {
    pub fn build(raw_data: &[u8]) -> Result<Self, DecoderError> {
        if raw_data[0..8] != PNG_HEADER {
            return Err(DecoderError::NotPngFile);
        }

        let mut index = 8;
        let mut ancillary_chunks = Vec::with_capacity(3);

        let mut ihdr = Chunk::new();
        let mut idat = Vec::new();
        let mut plte = None;

        while let Ok(chunk) = Chunk::from(&raw_data[index..]) {
            index += chunk.size;
            match chunk.ctype.as_str() {
                "IHDR" => ihdr = chunk,
                "IDAT" => idat.push(chunk),
                "PLTE" => plte = Some(chunk),
                _ => ancillary_chunks.push(chunk),
            }
        }

        Ok(Self {
            raw_data: raw_data.to_vec(),
            ihdr,
            plte,
            idat,
            ancillary_chunks,
        })
    }
}

#[derive(Debug)]
pub enum DecoderError {
    NotPngFile,
    TypeError(String),
    IoError(io::Error),
    NoMoreChunks(usize),
    InvalidChunk(&'static str),
}

// Defines how DecoderErrors are displayed.
impl Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::NotPngFile => {
                write!(f, "Error: File is not a valid PNG file.")
            }
            DecoderError::TypeError(e) => {
                write!(
                    f,
                    "Error: Attempted to decode incompatible type as png, '{e}'."
                )
            }
            DecoderError::IoError(e) => {
                write!(f, "Error: decoder cause an io::Error, '{e}'")
            }
            DecoderError::NoMoreChunks(v) => {
                write!(f, "Error: No more chunks left to iterate over, reached end of file at index '{v}'")
            }
            DecoderError::InvalidChunk(s) => {
                write!(f, "Error: Invalid chunk, {}", s)
            }
        }
    }
}

// Allows for conversion from io::Error to DecoderError.
impl From<io::Error> for DecoderError {
    fn from(error: io::Error) -> Self {
        DecoderError::IoError(error)
    }
}

// Implements the Error interface for CliError.
impl Error for DecoderError {}
