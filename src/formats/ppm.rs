use std::{fs, io, path::Path};
pub struct PpmSmall {
    pub header: Vec<u8>,
    pub dimensions: (u8, u8),
    pub data: Vec<(u8, u8, u8)>,
}

impl PpmSmall {
    pub fn build(data: &[(u8, u8, u8)], x: u8, y: u8) -> Self {
        let header = vec![
            0x50,
            0x36,
            0x0a,
            x + 48,
            0x20,
            y + 48,
            0x0a,
            0x32,
            0x35,
            0x35,
            0x0a,
        ];
        Self {
            header,
            dimensions: (x, y),
            data: data.to_vec(),
        }
    }
    pub fn write<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file_bytes = Vec::with_capacity(self.data.len());

        file_bytes.extend_from_slice(&self.header);

        for pixel in &self.data {
            file_bytes.extend_from_slice(&[pixel.0, pixel.1, pixel.2]);
        }

        file_bytes.push(0x0a);

        fs::write(path, &file_bytes)?;
        Ok(())
    }
}
