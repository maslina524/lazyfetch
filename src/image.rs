use alloc::{vec, vec::Vec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    Rgba,
    Rgb,
    GrayscaleAlpha,
    Grayscale,
}

impl ColorType {
    pub const fn get_bytes_count(self) -> usize {
        match self {
            Self::Rgba => 4,
            Self::Rgb => 3,
            Self::GrayscaleAlpha => 2,
            Self::Grayscale => 1,
        }
    }
}

impl TryFrom<u8> for ColorType {
    type Error = ColorTypeError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Grayscale),
            2 => Ok(Self::Rgb),
            4 => Ok(Self::GrayscaleAlpha),
            6 => Ok(Self::Rgba),
            _ => Err(ColorTypeError(value)),
        }
    }
}

#[derive(Debug)]
pub struct ColorTypeError(u8);

impl core::fmt::Display for ColorTypeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ColorTypeError: Failed to get ColorType from index ({}); file is corrupted or lazyfetch does not support the type, try converting the file to another type",
            self.0
        )
    }
}

impl core::error::Error for ColorTypeError {}

#[derive(Debug)]
pub struct RgbaConvertError {
    pub typ: ColorType,
    pub bits: u8,
}

impl core::fmt::Display for RgbaConvertError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RgbaConvertError: lazyfetch does not support {:?} with {}-bit color depth, try converting the image to another format",
            self.typ, self.bits
        )
    }
}

impl core::error::Error for RgbaConvertError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const fn from(data: &[u8], typ: ColorType, bits: u8) -> Result<Self, RgbaConvertError> {
        match (typ, bits) {
            (ColorType::Rgba, 8) => Ok(Self(data[0], data[1], data[2], data[3])),
            (ColorType::Rgba, 16) => {
                let r = u16::from_be_bytes([data[0], data[1]]);
                let g = u16::from_be_bytes([data[2], data[3]]);
                let b = u16::from_be_bytes([data[4], data[5]]);
                let a = u16::from_be_bytes([data[6], data[7]]);
                Ok(Self(
                    (r >> 8) as u8,
                    (g >> 8) as u8,
                    (b >> 8) as u8,
                    (a >> 8) as u8,
                ))
            }
            (ColorType::Rgb, 8) => Ok(Self(data[0], data[1], data[2], 255)),
            (ColorType::Rgb, 16) => {
                let r = u16::from_be_bytes([data[0], data[1]]);
                let g = u16::from_be_bytes([data[2], data[3]]);
                let b = u16::from_be_bytes([data[4], data[5]]);
                Ok(Self((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8, 255))
            }
            (ColorType::Grayscale, 8) => {
                let g = data[0];
                Ok(Self(g, g, g, 255))
            }
            (ColorType::Grayscale, 16) => {
                let g = u16::from_be_bytes([data[0], data[1]]);
                let g8 = (g >> 8) as u8;
                Ok(Self(g8, g8, g8, 255))
            }
            (ColorType::Grayscale, 1 | 2 | 4) => {
                let val = data[0];
                let g = match bits {
                    1 => {
                        if val == 0 {
                            0
                        } else {
                            255
                        }
                    }
                    2 => val * 85,
                    4 => val * 17,
                    _ => unreachable!(),
                };
                Ok(Self(g, g, g, 255))
            }
            (ColorType::GrayscaleAlpha, 8) => {
                let g = data[0];
                let a = data[1];
                Ok(Self(g, g, g, a))
            }
            (ColorType::GrayscaleAlpha, 16) => {
                let g = u16::from_be_bytes([data[0], data[1]]);
                let a = u16::from_be_bytes([data[2], data[3]]);
                let g8 = (g >> 8) as u8;
                let a8 = (a >> 8) as u8;
                Ok(Self(g8, g8, g8, a8))
            }

            _ => Err(RgbaConvertError { typ, bits }),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    w: usize,
    h: usize,
    data: Vec<Rgba>,
}

impl Image {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            data: vec![Rgba::default(); w * h],
        }
    }

    pub const fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, v: Rgba) -> Option<()> {
        if x < self.w && y < self.h {
            self.data[y * self.w + x] = v;
            Some(())
        } else {
            None
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Rgba> {
        self.data.get(y * self.w + x).copied()
    }

    pub fn get_pixel_wrapped(&self, x: isize, y: isize) -> Option<Rgba> {
        if x < 0 || y < 0 || x >= self.w as isize || y >= self.h as isize {
            return None;
        }
        self.data.get(y as usize * self.w + x as usize).copied()
    }

    pub fn get_pixel_wrapped_or_default(&self, x: isize, y: isize) -> Rgba {
        self.get_pixel_wrapped(x, y).unwrap_or_default()
    }

    pub const fn get_size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn as_rgba_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.w * self.h * 4);
        for p in &self.data {
            out.extend_from_slice(&[p.0, p.1, p.2, p.3]);
        }
        out
    }
}
