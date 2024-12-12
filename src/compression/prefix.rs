use std::collections::BTreeMap;

/// Code lengths from section 3.2.6 of RFC 1951.
pub const FIXED_CODE_LENGTHS: [u8; 288] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
];

/// The number of extra bits each length code has.
pub const LENGTH_EXTRA_BITS: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

/// The base length value of each length code.
pub const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

/// The number of extra bits each distance code has.
pub const DISTANCE_EXTRA_BITS: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The base value of each distance code.
pub const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

/// A struct for representing codes of differing bit lengths, codes are stored
/// little endian, meant to be read from most significant bit to least
/// significant bit.
///
/// # Fields
///
/// * 'buffer' - A u32 acting as a bit buffer.
/// * 'length' - A u8 specifying how many of the bits in the buffer are actually
///         a part of the code. A length of 2 would mean that the 2 least
///         significant bits hold the code.
///
/// # Methods
///
/// * 'new' - Generates a new empty Code.
/// * 'from' - Accepts a buffer and a length and creates a Code struct from
///         those given values.
/// * 'push' - Accepts a buffer and a length and pushes length bits of value
///         into the bit buffer.
/// * 'push_bit' - Accepts a single u8 which is normalized to represent either
///         a 0 or 1, and pushes it to the buffer.
///
/// # Examples
///
/// '''
/// let new = Code::new();
/// let from = Code::from(0b1011, 4);
///
/// new_code.push_bit(1);
/// new_code.push(0b011, 3);
///
/// // Both codes now have a length of 4, and the u32 value:
/// // 0b0000_0000_0000_0000_0000_0000_0000_1011
/// assert_eq!(new.code, from.code);
/// '''
#[derive(Clone, Debug)]
pub struct Code {
    pub buffer: u16,
    pub length: u8,
    index: u8,
}

impl Code {
    /// Constructs a new, empty instance of Code.
    ///
    /// # Returns
    ///
    /// A Code struct with zeroes for all fields.
    pub fn new() -> Self {
        Self {
            buffer: 0,
            length: 0,
            index: 0,
        }
    }
    /// Constructs an instance of Code with a given code and length.
    ///
    /// # Arguments
    ///
    /// * 'code' - The u32 value containing the binary code.
    /// * 'length' - A u8 representing the number of bits of code are a part
    ///         of the binary code.
    ///
    /// # Returns
    ///
    /// A Code struct with the given values.
    pub fn from(buffer: u16, length: u8) -> Self {
        Self {
            buffer,
            length,
            index: 0,
        }
    }
    /// Accepts a length and a u32 as a buffer, and pushes length bits of that
    /// buffer into self.code and increments self.length by the appropriate
    /// amount.
    ///
    /// # Arguments
    ///
    /// * 'buffer' - A u32 acting as a bit buffer containing the bits to push.
    /// * 'length' - The number of bits to push.
    pub fn push(&mut self, buffer: u16, length: u8) {
        self.buffer = (self.buffer << length) | buffer;
        self.length += length;
    }
    /// Accepts either a 0 or 1 and pushes that bit to self. If a non-binary
    /// value is entered it will correct it to a 1 instead of raising an error.
    ///
    /// # Arguments
    ///
    /// * 'bit' - A u8 representing the bit to push.
    pub fn push_bit(&mut self, bit: u8) {
        let normalized_bit: u16 = match bit {
            0 => 0,
            1 => 1,
            _ => {
                eprintln!("Warning: Non-binary value passed to push, value corrected to a 1.");
                1
            }
        };

        self.buffer = (self.buffer << 1) | normalized_bit;
        self.length += 1;
    }
}

impl Default for Code {
    fn default() -> Self {
        Code::new()
    }
}

impl Iterator for Code {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            self.index += 1;
            Some((self.buffer >> (self.length - self.index) & 1) as u8)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0length$b}",
            self.buffer,
            length = self.length as usize
        )
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.eq(&other.buffer) && self.length.eq(&other.length)
    }
}

impl Eq for Code {}

impl Ord for Code {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.buffer
            .cmp(&other.buffer)
            .then_with(|| self.length.cmp(&other.length))
    }
}

impl PartialOrd for Code {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct PrefixCodeMap {
    pub map: BTreeMap<Code, usize>,
}

impl PrefixCodeMap {
    pub fn from_lengths(code_lengths: &[u8]) -> Self {
        let mut occurances = [0u16; 256];
        let max_length = *code_lengths.iter().max().unwrap() as usize;

        code_lengths.iter().fold(&mut occurances, |acc, &idx| {
            (*acc)[idx as usize] = (*acc)[idx as usize].saturating_add(1);
            acc
        });

        let mut next_code = vec![0; max_length + 1];
        let mut code = 0;
        occurances[0] = 0;
        for i in 1..=max_length {
            code = (code + occurances[i - 1]) << 1;
            next_code[i] = code;
        }

        let mut codes = vec![None; code_lengths.len()];

        for j in 0..code_lengths.len() {
            let len = code_lengths[j] as usize;
            if len != 0 {
                codes[j] = Some(next_code[len]);
                next_code[len] += 1;
            }
        }

        let mut map = BTreeMap::new();

        for (index, code) in codes.iter().enumerate() {
            if let Some(c) = code {
                let code_struct = Code::from(c.to_owned(), code_lengths[index]);
                map.insert(code_struct, index);
            }
        }

        Self { map }
    }
}
