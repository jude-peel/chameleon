use std::{
    error::Error,
    fmt::{self, Display},
    fs, io,
    path::Path,
    str,
};

use crate::compression::{crc, zlib::ZlibStream};

// +-----------+
// | CONSTANTS |
// +-----------+

pub const PNG_HEADER: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

const VALID_CHUNK_TYPES: [&str; 18] = [
    "IHDR", "PLTE", "IDAT", "IEND", "cHRM", "gAMA", "iCCP", "sBIT", "sRGB", "bKGD", "hIST", "tRNS",
    "pHYs", "sPLT", "tIME", "iTXt", "tEXt", "zTXt",
];

//      +------------------+
//      | PNG OPTION ENUMS |
//      +------------------+

pub enum ColorType {
    Grayscale,
    RGB,
    PalleteIndex,
    GrayscaleAlpha,
    RGBA,
}

pub enum Interlace {
    None,
    Adam7,
}

//      +-------------+
//      | FILE FORMAT |
//      +-------------+

/// A structure containing a PNG file.
///
/// # Fields
///
/// * 'data'
/// * 'dimensions' -
/// * 'bit_depth' -
/// * 'color_type' -
/// * 'interlace' -
///
/// # Examples
///
/// '''
/// // Open the file.
/// let mut png = Png::from_path("./example.png")?;
///
/// // Decode the file into a vector of rgb tuples.
/// let mut rgb_vec = png.rgb()?;
///
/// // Decrease the level of red in the image.
/// for rgb in rgb_vec.iter_mut() {
///     *rgb.0.saturating_sub(50);
/// }
/// '''
pub struct Png {
    pub data: PngData,
    pub dimensions: (usize, usize),
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub interlace: Interlace,
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
    ///
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Png, DecoderError> {
        let path = path.as_ref();

        let file_bytes = fs::read(path)?;

        if file_bytes[0..8] != PNG_HEADER {
            return Err(DecoderError::NotPngFile);
        }

        let data = PngData::build(&file_bytes)?;

        let dimensions = (
            data.ihdr.data[0..4]
                .iter()
                .fold(0usize, |acc, byte| (acc << 8) | *byte as usize),
            data.ihdr.data[4..8]
                .iter()
                .fold(0usize, |acc, byte| (acc << 8) | *byte as usize),
        );

        let bit_depth = data.ihdr.data[8];

        let color_type = match data.ihdr.data[9] {
            0 => ColorType::Grayscale,
            2 => ColorType::RGB,
            3 => ColorType::PalleteIndex,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
            other => {
                return Err(DecoderError::InvalidColorType(other));
            }
        };

        let interlace = match data.ihdr.data[12] {
            0 => Interlace::None,
            1 => Interlace::Adam7,
            other => {
                return Err(DecoderError::InvalidInterlace(other));
            }
        };

        //let interlace = matches!()

        Ok(Png {
            data,
            dimensions,
            bit_depth,
            color_type,
            interlace,
        })
    }
    /// Converts the PNG file into a vector of rgb tuples.
    ///
    /// # Returns
    ///
    /// A Vec<(u8, u8, u8)> containing each pixel from left to right, top to
    /// bottom. Remember to store the dimensions for future encoding.
    ///
    pub fn rgb(&self) -> Vec<(u8, u8, u8)> {
        // Concatenate the data from all IDAT chunks.
        let zlib_bytes = self
            .data
            .idat
            .iter()
            .flat_map(|ch| &ch.data)
            .cloned()
            .collect::<Vec<_>>();

        let mut zlib = ZlibStream::build(&zlib_bytes).unwrap();

        let data = zlib.decompress().unwrap();

        // Get the number of samples per pixel.
        let samples: usize = match self.color_type {
            ColorType::Grayscale => 1,
            ColorType::RGB => 3,
            ColorType::PalleteIndex => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGBA => 4,
        };

        let bpp = samples as u8 * (self.bit_depth / 8);
        println!("bpp: {}, bit_depth: {}", bpp, self.bit_depth);

        // Split the data into each individual scanline.
        let scanlines = data
            .chunks((samples * self.dimensions.0) + 1)
            .collect::<Vec<_>>();

        let mut last = vec![0u8; samples * self.dimensions.0];

        let mut defiltered_scanlines: Vec<Vec<u8>> = Vec::with_capacity(scanlines.len());

        for scanline in scanlines {
            println!("{}", scanline[0]);
            match scanline[0] {
                0 => defiltered_scanlines.push(scanline[1..].to_vec()),
                1 => {
                    defiltered_scanlines.push(rfsub(&scanline[1..], bpp as usize));
                }
                2 => {
                    defiltered_scanlines.push(rfup(&scanline[1..], &last));
                }
                3 => {
                    defiltered_scanlines.push(rfaverage(&scanline[1..], &last, bpp as usize));
                }
                4 => {
                    defiltered_scanlines.push(rfpaeth(&scanline[1..], &last, bpp as usize));
                }
                _ => {}
            }
            last = defiltered_scanlines.last().unwrap().clone();
        }

        let mut output = Vec::new();

        for line in defiltered_scanlines {
            for values in line.chunks(3) {
                output.push((values[0], values[1], values[2]));
            }
        }

        output
    }
}

/// A structure for representing each individual chunk in the PNG file mostly for
/// internal use. These chunks have a header containing the length of the data
/// in the chunk as a u32, a 4 byte type, the actual data of the chunk, then
/// a CRC32 checksum.
///
/// # Fields
///
/// * 'length' - The length of the data in the chunk.
/// * 'ctype' - The type of the chunk.
/// * 'data' - The data held within the chunk.
/// * 'crc' - The CRC32 checksum.
/// * 'size' - The overall size of the chunk (including the header and checksum).
///
#[derive(Clone, Debug)]
pub struct Chunk {
    pub length: usize,
    pub ctype: String,
    pub data: Vec<u8>,
    pub crc: u32,
    pub size: usize,
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
///
#[derive(Debug)]
pub struct PngData {
    pub raw_data: Vec<u8>,
    pub ihdr: Chunk,
    pub plte: Option<Chunk>,
    pub idat: Vec<Chunk>,
    pub ancillary_chunks: Vec<Chunk>,
}

impl PngData {
    /// Parses the raw bytes of a PNG file and organizes it into chunks.
    ///
    /// # Arguments
    ///
    /// * 'raw_data' - A slice containing the entire PNG file as bytes.
    ///
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

//      +---------+
//      | FILTERS |
//      +---------+

/// Enum for storing each filter type described in Chapter 6 of the spec.
/// Each filter is defined by a single byte before each scanline, and
/// applies to each byte, regardless of bit depth. Most pixels have more
/// than one bytes worth of information, and so in these cases, the filter
/// is applied referencing the corrosponding byte of the previous pixel.
/// So, if the color type is RGB with a bit-depth of 8, each sample for
/// red would be filtered together, and then each sample for blue, and so
/// on.
///
/// # Members
///
/// * 'None' - No filter is applied.
/// * 'Sub' - Each byte transmits the difference between itself and the last
///         corrosponding byte.
/// * 'Up' - Each byte is the same as sub however it transmits the difference
///         between the current byte and the corrosponding byte from the pixel
///         directly above it (same position in the previous scanline).
/// * 'Average' - Subtracts the average of the bytes in the pixels to the left
///         and above from the current byte.
/// * 'Paeth' - A bit too complex to be worth summarizing, it's described in
///         section 6.6 of the specification.
///         
pub enum Filters {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

pub fn rfsub(scanline: &[u8], bpp: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(scanline.len());
    for (i, &byte) in scanline.iter().enumerate() {
        let left = if i >= bpp { buf[i - bpp] } else { 0 };

        buf.push(byte.wrapping_add(left));
    }

    buf
}

pub fn rfup(scanline: &[u8], last: &[u8]) -> Vec<u8> {
    scanline
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte.wrapping_add(last[i]))
        .collect()
}

pub fn rfaverage(scanline: &[u8], last: &[u8], bpp: usize) -> Vec<u8> {
    scanline
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            let left = if i >= bpp { scanline[i - bpp] } else { 0 };
            let above = last[i];
            byte.wrapping_add((left + above) / 2)
        })
        .collect()
}

pub fn rfpaeth(scanline: &[u8], last: &[u8], bpp: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(scanline.len());

    for (i, &byte) in scanline.iter().enumerate() {
        let left = if i >= bpp { buf[i - bpp] } else { 0 };
        let above = last[i];
        let upper_left = if i >= bpp { last[i - bpp] } else { 0 };

        buf.push(byte.wrapping_add(fpaeth(left, above, upper_left)));
    }
    println!("{:?}", buf);
    buf
}

pub fn fpaeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let (a, b, c) = (left as i16, above as i16, upper_left as i16);
    let p = (a + b) - c;
    let pa = p.abs_diff(a);
    let pb = p.abs_diff(b);
    let pc = p.abs_diff(c);

    if pa <= pb && pa <= pc {
        left
    } else if pb <= pc {
        above
    } else {
        upper_left
    }
}

//      +----------------+
//      | ERROR CHECKING |
//      +----------------+

/// Custom error types for decoder related errors.
///
/// # Members
///
/// * 'NotPngFile' - Pretty self-explainatory, used when the file given is not
///         a valid PNG file. Called when the input file either lacks the .png
///         extension, or does not have the PNG header.
/// * 'IoError' - A wrapper for the std::io::Error type to be called when the
///         decoder tries something that causes an io::Error.
/// * 'InvalidChunk' - Called when the chunk being parsed is not valid, either
///         because the header is incorrect, or the CRC32 deos not match the
///         data. Holds a &str for communicating why the chunk is invalid.
/// * 'InvalidColorType' - Used if the byte for color type in IHDR is not set
///         to either of the valid types: 1, 2, 3, 4, or 6. Holds the invalid
///         color type byte.
/// * 'InvalidInterlace' - Used if the byte for the interlace is invalid (not
///         0 or 1). Holds the invalid interlace byte.
///         
#[derive(Debug)]
pub enum DecoderError {
    NotPngFile,
    IoError(io::Error),
    InvalidChunk(&'static str),
    InvalidColorType(u8),
    InvalidInterlace(u8),
}

// Defines how DecoderErrors are displayed.
impl Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::NotPngFile => {
                write!(f, "Error: File is not a valid PNG file.")
            }
            DecoderError::IoError(e) => {
                write!(f, "Error: The decoder caused an io::Error, '{e}'")
            }
            DecoderError::InvalidChunk(s) => {
                write!(f, "Error: Invalid chunk, {}", s)
            }
            DecoderError::InvalidColorType(t) => {
                write!(
                    f,
                    "Error: Invalid color type {}, see PNG Specification 4.1.1 for valid types.",
                    t
                )
            }
            DecoderError::InvalidInterlace(i) => {
                write!(f, "Error: Invalid interlace value {}, only 0 (none) or 1 (Adam7 interlace) are currently valid.", i)
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
