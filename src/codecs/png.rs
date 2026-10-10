use core::error::Error;

use alloc::{vec, vec::Vec};

use crate::{
    crc32,
    deflate::{self, DeflateError},
    image::{ColorType, ColorTypeError, Image, Rgba, RgbaConvertError},
};

static PNG_SIG: &[u8] = b"\x89PNG\x0D\x0A\x1A\x0A";
static IHDR_NAME: &[u8] = b"IHDR";

#[derive(Debug)]
pub enum PngError {
    InvalidSignature,
    IncorrectDataLen,
    FirstChunkIsNotIHDR,
    IncorrectIHDRLen,
    IncorrectIHDRName,
    UnsupportedCompression,
    UnsupportedFilter,
    UnsupportedInterlace,
    UnexpectedEof,
    IncorrectCrc32,
    DeflateError(DeflateError),
    ColorTypeError(ColorTypeError),
    RgbaConvertError(RgbaConvertError),
}

impl core::fmt::Display for PngError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Png: Invalid signature"),
            Self::IncorrectDataLen => write!(f, "Png: Incorrect data len"),
            Self::FirstChunkIsNotIHDR => write!(f, "Png: First chunk is not IHDR"),
            Self::IncorrectIHDRLen => write!(f, "Png: Incorrect IHDR len"),
            Self::IncorrectIHDRName => write!(f, "Png: Incorrect IHDR name"),
            Self::UnsupportedCompression => {
                write!(f, "Png: Unsupported compression method (only Deflate)")
            }
            Self::UnsupportedFilter => write!(f, "Png: Unsupported filter method (only 0)"),
            Self::UnsupportedInterlace => write!(f, "Png: Unsupported interlace method (only 0)"),
            Self::UnexpectedEof => write!(f, "Png: Unexpected end of data"),
            Self::IncorrectCrc32 => write!(f, "Png: Chunk has invalid crc32, file is corrupted"),
            Self::DeflateError(e) => write!(f, "{e}"),
            Self::ColorTypeError(e) => write!(f, "{e}"),
            Self::RgbaConvertError(e) => write!(f, "{e}"),
        }
    }
}

impl From<DeflateError> for PngError {
    fn from(value: DeflateError) -> Self {
        Self::DeflateError(value)
    }
}

impl From<ColorTypeError> for PngError {
    fn from(value: ColorTypeError) -> Self {
        Self::ColorTypeError(value)
    }
}

impl From<RgbaConvertError> for PngError {
    fn from(value: RgbaConvertError) -> Self {
        Self::RgbaConvertError(value)
    }
}

impl Error for PngError {}

#[derive(Debug)]
struct Ihdr {
    pub width: usize,
    pub height: usize,
    pub depth: u8,
    pub color_type: ColorType,
    pub compression: u8,
    pub filter: u8,
    pub interlace: u8,
}

impl Ihdr {
    pub fn new<I>(iter: &mut I) -> Result<Self, PngError>
    where
        I: Iterator<Item = u8>,
    {
        let len = u32_from_bytes(&safe_take(iter, 4)?);
        if len != 13 {
            return Err(PngError::IncorrectIHDRLen);
        }

        let name = safe_take(iter, 4)?;
        if name != IHDR_NAME {
            return Err(PngError::IncorrectIHDRName);
        }

        let raw = safe_take(iter, 13)?;

        let width = u32_from_bytes(&raw[0..4]) as usize;
        let height = u32_from_bytes(&raw[4..8]) as usize;
        let depth = raw[8];
        let color_type_u8 = raw[9];
        let color_type = ColorType::try_from(color_type_u8)?;
        let compression = raw[10];
        let filter = raw[11];
        let interlace = raw[12];

        if compression != 0 {
            return Err(PngError::UnsupportedCompression);
        }
        if filter != 0 {
            return Err(PngError::UnsupportedFilter);
        }
        if interlace != 0 {
            return Err(PngError::UnsupportedInterlace);
        }

        let crc = u32_from_bytes(&safe_take(iter, 4)?);
        let mut crc_data = name;
        crc_data.extend_from_slice(&raw);

        let my_crc = crc32::encrypt_png(&crc_data);
        if crc != my_crc {
            return Err(PngError::IncorrectCrc32);
        }

        Ok(Self {
            width,
            height,
            depth,
            color_type,
            compression,
            filter,
            interlace,
        })
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub name: [u8; 4],
    pub data: Vec<u8>,
}

impl Chunk {
    pub fn new<I>(iter: &mut I) -> Result<Self, PngError>
    where
        I: Iterator<Item = u8>,
    {
        let len = u32_from_bytes(&safe_take(iter, 4)?);
        let name_bytes = safe_take(iter, 4)?;
        let mut name = [0u8; 4];
        name.copy_from_slice(&name_bytes);

        let raw = safe_take(iter, len as usize)?;

        let crc = u32_from_bytes(&safe_take(iter, 4)?);
        let mut crc_data = name_bytes;
        crc_data.extend_from_slice(&raw);

        let my_crc = crc32::encrypt_png(&crc_data);
        if crc != my_crc {
            return Err(PngError::IncorrectCrc32);
        }

        Ok(Self { name, data: raw })
    }
}

fn safe_take<I>(iter: &mut I, n: usize) -> Result<Vec<I::Item>, PngError>
where
    I: Iterator<Item = u8>,
{
    let taken: Vec<u8> = iter.by_ref().take(n).collect();
    if taken.len() == n {
        Ok(taken)
    } else {
        Err(PngError::IncorrectDataLen)
    }
}

const fn u32_from_bytes(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

const fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn unfilter_scanline(
    filter: u8,
    current: &mut [u8],
    previous: Option<&[u8]>,
    bpp: usize,
) -> Result<(), PngError> {
    for i in 0..current.len() {
        let a = if i >= bpp { current[i - bpp] } else { 0 };
        let b = previous.map_or(0, |p| p[i]);
        let c = if i >= bpp {
            previous.map_or(0, |p| p[i - bpp])
        } else {
            0
        };

        current[i] = match filter {
            0 => current[i],
            1 => current[i].wrapping_add(a),
            2 => current[i].wrapping_add(b),
            3 => current[i].wrapping_add(u16::midpoint(a as u16, b as u16) as u8),
            4 => current[i].wrapping_add(paeth_predictor(a, b, c)),
            _ => return Err(PngError::UnsupportedFilter),
        };
    }
    Ok(())
}

fn extract_pixel_bytes(row: &[u8], x: usize, color_type: ColorType, depth: u8) -> Vec<u8> {
    let channels = color_type.get_bytes_count();
    if depth >= 8 {
        let bytes_per_pixel = channels * (depth as usize / 8);
        let start = x * bytes_per_pixel;
        row[start..start + bytes_per_pixel].to_vec()
    } else {
        let pixels_per_byte = 8 / depth as usize;
        let byte_index = x / pixels_per_byte;
        let bit_index = x % pixels_per_byte;
        let shift = 8 - depth as usize * (bit_index + 1);
        let mask = ((1u16 << depth) - 1) as u8;
        vec![(row[byte_index] >> shift) & mask]
    }
}

pub fn is_png(bytes: &[u8]) -> bool {
    if bytes.len() < PNG_SIG.len() {
        return false;
    }
    &bytes[..PNG_SIG.len()] == PNG_SIG
}

#[derive(Debug)]
pub struct Png {
    image: Image,
    typ: ColorType,
    depth: u8,
}

impl Png {
    pub fn decode(bytes: &[u8]) -> Result<Self, PngError> {
        let mut iter = bytes.iter().copied();

        let sig = safe_take(&mut iter, PNG_SIG.len())?;
        if !is_png(&sig) {
            return Err(PngError::InvalidSignature);
        }

        let ihdr = Ihdr::new(&mut iter)?;
        // crate::println!("IHDR: {:?}", ihdr);

        let mut idat_data = Vec::with_capacity(8 * 1024);
        loop {
            let chunk = Chunk::new(&mut iter)?;
            // crate::println!("{}: {:?}", str::from_utf8(&chunk.name).unwrap(), chunk.data);
            match &chunk.name {
                b"IEND" => {
                    break;
                }
                b"IDAT" => {
                    idat_data.extend_from_slice(&chunk.data);
                }
                _ => {}
            }
        }

        let deflate_raw = &idat_data[2..idat_data.len() - 4];
        let decompressed = deflate::decode(deflate_raw)?;
        // crate::println!("decompressed IDAT: {:?}", decompressed);
        let mut iter = decompressed.iter().copied();

        let channels = ihdr.color_type.get_bytes_count();
        let bpp = core::cmp::max(1, channels * ihdr.depth as usize / 8);
        let row_bytes = (ihdr.width * channels * ihdr.depth as usize).div_ceil(8);

        let mut image = Image::new(ihdr.width, ihdr.height);
        let mut prev_row = None;

        for y in 0..ihdr.height {
            let filter = iter.next().ok_or(PngError::UnexpectedEof)?;
            let mut row = safe_take(&mut iter, row_bytes)?;

            unfilter_scanline(filter, &mut row, prev_row.as_deref(), bpp)?;

            for x in 0..ihdr.width {
                let pixel_raw = extract_pixel_bytes(&row, x, ihdr.color_type, ihdr.depth);
                let pixel = Rgba::from(&pixel_raw, ihdr.color_type, ihdr.depth)?;
                image.set_pixel(x, y, pixel).unwrap();
            }

            prev_row = Some(row);
        }

        Ok(Self {
            image,
            typ: ihdr.color_type,
            depth: ihdr.depth,
        })
    }

    pub const fn as_image(&self) -> &Image {
        &self.image
    }

    pub fn into_image(self) -> Image {
        self.image
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{codecs::Png, image::ColorType};

    #[test]
    fn rgba8_test() {
        let data = fs::read("test/png/rgba8.png").unwrap();
        let png = Png::decode(&data).unwrap();
        let size = png.as_image().size();

        println!("{png:#?}");
        assert_eq!(size, (3, 3));
        assert_eq!(png.depth, 8);
        assert_eq!(png.typ, ColorType::Rgba);
    }

    #[test]
    fn graya16_test() {
        let data = fs::read("test/png/gray16.png").unwrap();
        let png = Png::decode(&data).unwrap();
        let size = png.as_image().size();

        println!("{png:#?}");
        assert_eq!(size, (2, 2));
        assert_eq!(png.depth, 16);
        assert_eq!(png.typ, ColorType::GrayscaleAlpha);
    }

    #[test]
    fn gray8_test() {
        let data = fs::read("test/png/gray8.png").unwrap();
        let png = Png::decode(&data).unwrap();
        let size = png.as_image().size();

        println!("{png:#?}");
        assert_eq!(size, (2, 2));
        assert_eq!(png.depth, 8);
        assert_eq!(png.typ, ColorType::Grayscale);
    }
}
