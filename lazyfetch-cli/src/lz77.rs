use core::error::Error;

use alloc::vec::Vec;

#[derive(Debug)]
pub enum Lz77Error {
    ReachedEndOfBuffer,
    UnknownControlByte(u8),
    OffseLargerThanDict,
    IndexOutOfBounds,
}

impl core::fmt::Display for Lz77Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReachedEndOfBuffer => write!(f, "Lz77: Reached end of buffer"),
            Self::UnknownControlByte(b) => write!(f, "Lz77: Unknow control byte ({b:02X})"),
            Self::OffseLargerThanDict => write!(f, "Lz77: Offset larger than dictionary"),
            Self::IndexOutOfBounds => write!(f, "Lz77: Index out of bounds"),
        }
    }
}

impl Error for Lz77Error {}

pub fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>, Lz77Error> {
    let cap = bytes.len();
    let mut iter = bytes.into_iter();
    let mut dictionary = Vec::with_capacity(cap);

    while let Ok(offset) = get_control_bytes(&mut iter) {
        match offset {
            Offset::Dictionary { length, offset } => {
                let dict = fetch_offset(&dictionary, length, offset)?;
                dictionary.extend_from_slice(&dict);
            }
            Offset::Literal { length } => match safe_take(&mut iter, length) {
                Ok(mut bytes) => {
                    dictionary.append(&mut bytes);
                }
                Err(_) => {
                    return Err(Lz77Error::ReachedEndOfBuffer);
                }
            },
        }
    }

    Ok(dictionary)
}

#[derive(Debug, PartialEq)]
enum Offset {
    Literal { length: usize },
    Dictionary { length: usize, offset: usize },
}

fn fetch_offset(dictionary: &[u8], length: usize, offset: usize) -> Result<Vec<u8>, Lz77Error> {
    if offset > dictionary.len() {
        return Err(Lz77Error::OffseLargerThanDict);
    }

    let mut result = Vec::with_capacity(length);

    for i in 0..length {
        let pos = dictionary.len() - offset + (i % offset);
        if pos >= dictionary.len() {
            return Err(Lz77Error::IndexOutOfBounds);
        }
        result.push(dictionary[pos]);
    }

    Ok(result)
}

fn get_control_bytes<I>(reader: &mut I) -> Result<Offset, Lz77Error>
where
    I: Iterator<Item = u8>,
{
    let cb = next(reader)?;
    let q = q_mask(cb) as usize;
    let cb_mask = cb_mask(cb) as usize;

    Ok(match cb_mask {
        1 => Offset::Literal { length: 1 + q },
        3..=8 => {
            let r = next(reader)?;
            Offset::Dictionary {
                length: cb_mask,
                offset: ((q << 8) + r as usize + 1),
            }
        }

        9 => {
            let r = next(reader)?;
            let s = next(reader)?;

            Offset::Dictionary {
                length: 9 + r as usize,
                offset: ((q << 8) + s as usize + 1),
            }
        }
        _ => unreachable!(),
    })
}

const fn cb_mask(i: u8) -> u8 {
    [1, 3, 4, 5, 6, 7, 8, 9][((i >> 5) & 0b111) as usize]
}

const fn q_mask(i: u8) -> u8 {
    i & 0b0001_1111
}

fn next<I>(iter: &mut I) -> Result<u8, Lz77Error>
where
    I: Iterator<Item = u8>,
{
    iter.next().ok_or(Lz77Error::ReachedEndOfBuffer)
}

fn safe_take<I>(iter: &mut I, n: usize) -> Result<Vec<u8>, Lz77Error>
where
    I: Iterator<Item = u8>,
{
    let taken: Vec<u8> = iter.by_ref().take(n).collect();

    if taken.len() == n {
        Ok(taken)
    } else {
        Err(Lz77Error::ReachedEndOfBuffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz77::decompress;

    #[test]
    fn literal_single_test() {
        let compressed = vec![0x00, b'A'];
        let decompressed = decompress(compressed).unwrap();
        assert_eq!(decompressed, vec![b'A']);
    }

    #[test]
    fn literal_two_test() {
        let compressed = vec![0x01, b'A', b'B'];
        let decompressed = decompress(compressed).unwrap();
        assert_eq!(decompressed, vec![b'A', b'B']);
    }

    #[test]
    fn literal_and_dict_test() {
        let compressed = vec![0x01, b'A', b'B', 0x20, 0x01];
        let decompressed = decompress(compressed).unwrap();
        assert_eq!(decompressed, vec![b'A', b'B', b'A', b'B', b'A']);
    }
}
