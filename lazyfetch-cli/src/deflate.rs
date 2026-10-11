use core::{cmp::Ordering, error::Error, iter::repeat_n};

use alloc::vec::Vec;

use crate::huffman::{HuffmanTree, Stream, decode_symb};

const LENGTH_BASE: [(usize, u8); 29] = [
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (11, 1),
    (13, 1),
    (15, 1),
    (17, 1),
    (19, 2),
    (23, 2),
    (27, 2),
    (31, 2),
    (35, 3),
    (43, 3),
    (51, 3),
    (59, 3),
    (67, 4),
    (83, 4),
    (99, 4),
    (115, 4),
    (131, 5),
    (163, 5),
    (195, 5),
    (227, 5),
    (258, 0),
];

const DIST_BASE: [(usize, u8); 30] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 1),
    (7, 1),
    (9, 2),
    (13, 2),
    (17, 3),
    (25, 3),
    (33, 4),
    (49, 4),
    (65, 5),
    (97, 5),
    (129, 6),
    (193, 6),
    (257, 7),
    (385, 7),
    (513, 8),
    (769, 8),
    (1025, 9),
    (1537, 9),
    (2049, 10),
    (3073, 10),
    (4097, 11),
    (6145, 11),
    (8193, 12),
    (12289, 12),
    (16385, 13),
    (24577, 13),
];

fn extend_at<T: Clone>(dst: &mut [T], offset: usize, src: impl IntoIterator<Item = T>) {
    for (slot, val) in dst[offset..].iter_mut().zip(src) {
        *slot = val;
    }
}

fn build_fixed_trees() -> (HuffmanTree, HuffmanTree) {
    let mut litlen_lens = [0u8; 288];
    for (i, item) in litlen_lens.iter_mut().enumerate() {
        *item = match i {
            i if (144..256).contains(&i) => 7,
            i if (256..280).contains(&i) => 9,
            _ => 8,
        }
    }

    let mut litlen_alphabet = [0u32; 288];
    extend_at(&mut litlen_alphabet, 0, 0..288);
    let litlen_tree = HuffmanTree::from_alphabet_and_bl_list(&litlen_alphabet, &litlen_lens);

    let dist_lens = [5u8; 32];
    let mut dist_alphabet = [0u32; 32];
    extend_at(&mut dist_alphabet, 0, 0..32);
    let dist_tree = HuffmanTree::from_alphabet_and_bl_list(&dist_alphabet, &dist_lens);

    (litlen_tree, dist_tree)
}

const CLEN_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Debug)]
pub enum DeflateError {
    InvalidBtype,
    InvalidNlen,
    DecodeError,
    InvalidSymbol,
    InvalidDistanceSymbol,
    DistanceTooFar,
    DynamicTreeError,
    EndOfStream,
}

impl core::fmt::Display for DeflateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidBtype => write!(f, "Deflate: Invalid byte"),
            Self::InvalidNlen => write!(f, "Deflate: Invalid nlen"),
            Self::DecodeError => write!(f, "Deflate: Decode error"),
            Self::InvalidSymbol => write!(f, "Deflate: Invalid symbol"),
            Self::InvalidDistanceSymbol => write!(f, "Deflate: Invalid distance symbol"),
            Self::DistanceTooFar => write!(f, "Deflate: Distance too far"),
            Self::DynamicTreeError => write!(f, "Deflate: Dynamic tree error"),
            Self::EndOfStream => write!(f, "Deflate: End of stream"),
        }
    }
}

impl Error for DeflateError {}

fn build_dynamic_trees(stream: &mut Stream) -> Result<(HuffmanTree, HuffmanTree), DeflateError> {
    let hlit = stream.read_bits(5) as usize + 257;
    let hdist = stream.read_bits(5) as usize + 1;
    let hclen = stream.read_bits(4) as usize + 4;

    let mut clen_lens = [0u8; 19];
    for i in 0..hclen {
        clen_lens[CLEN_ORDER[i]] = stream.read_bits(3) as u8;
    }

    let mut clen_alphabet = [0u32; 19];
    extend_at(&mut clen_alphabet, 0, 0..19);
    let clen_tree = HuffmanTree::from_alphabet_and_bl_list(&clen_alphabet, &clen_lens);

    let mut lens: Vec<u8> = Vec::with_capacity(hlit + hdist);
    while lens.len() < hlit + hdist {
        let sym = decode_symb(stream, &clen_tree).ok_or(DeflateError::DecodeError)?;
        match sym {
            0..=15 => lens.push(sym as u8),
            16 => {
                let repeat = stream.read_bits(2) as usize + 3;
                let last = *lens.last().ok_or(DeflateError::DecodeError)?;
                lens.extend(repeat_n(last, repeat));
            }
            17 => {
                let repeat = stream.read_bits(3) as usize + 3;
                lens.extend(repeat_n(0, repeat));
            }
            18 => {
                let repeat = stream.read_bits(7) as usize + 11;
                lens.extend(repeat_n(0, repeat));
            }
            _ => return Err(DeflateError::DynamicTreeError),
        }
    }

    let litlen_lens = &lens[..hlit];
    let dist_lens = &lens[hlit..hlit + hdist];

    let litlen_alphabet: Vec<u32> = (0..hlit as u32).collect();
    let dist_alphabet: Vec<u32> = (0..hdist as u32).collect();

    let litlen_tree = HuffmanTree::from_alphabet_and_bl_list(&litlen_alphabet, litlen_lens);
    let dist_tree = HuffmanTree::from_alphabet_and_bl_list(&dist_alphabet, dist_lens);

    Ok((litlen_tree, dist_tree))
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, DeflateError> {
    let mut stream = Stream::new(input);
    let mut output = Vec::with_capacity(input.len());

    loop {
        let bfinal = stream.read_bit();
        let btype = stream.read_bits(2) as u8;

        match btype {
            0 => {
                stream.align_to_byte();
                let len = stream.read_bytes(2) as usize;
                let nlen = stream.read_bytes(2) as usize;
                if (len ^ 0xFFFF) != nlen {
                    return Err(DeflateError::InvalidNlen);
                }
                for _ in 0..len {
                    output.push(stream.read_byte());
                }
            }
            1 => {
                let (litlen_tree, dist_tree) = build_fixed_trees();
                decode_block(&mut stream, &mut output, &litlen_tree, &dist_tree)?;
            }
            2 => {
                let (litlen_tree, dist_tree) = build_dynamic_trees(&mut stream)?;
                decode_block(&mut stream, &mut output, &litlen_tree, &dist_tree)?;
            }
            _ => {
                return Err(DeflateError::InvalidBtype);
            }
        }

        if bfinal == 1 {
            break;
        }
    }

    Ok(output)
}

fn decode_block(
    stream: &mut Stream,
    output: &mut Vec<u8>,
    litlen_tree: &HuffmanTree,
    dist_tree: &HuffmanTree,
) -> Result<(), DeflateError> {
    loop {
        let sym = decode_symb(stream, litlen_tree).ok_or(DeflateError::DecodeError)?;

        match sym.cmp(&256) {
            Ordering::Less => output.push(sym as u8),
            Ordering::Greater => {
                let idx = (sym - 257) as usize;
                if idx >= LENGTH_BASE.len() {
                    return Err(DeflateError::InvalidSymbol);
                }
                let (base, extra_bits) = LENGTH_BASE[idx];
                let length = base + stream.read_bits(extra_bits as usize) as usize;

                let dist_sym = decode_symb(stream, dist_tree).ok_or(DeflateError::DecodeError)?;
                if dist_sym >= 30 {
                    return Err(DeflateError::InvalidDistanceSymbol);
                }
                let (dist_base, dist_extra) = DIST_BASE[dist_sym as usize];
                let distance = dist_base + stream.read_bits(dist_extra as usize) as usize;

                if distance > output.len() {
                    return Err(DeflateError::DistanceTooFar);
                }

                let start = output.len() - distance;
                for i in 0..length {
                    output.push(output[start + i]);
                }
            }
            Ordering::Equal => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::deflate::decode;

    #[test]
    fn decode_test() {
        let encoded = b"\xf3H\xcd\xc9\xc9WpIM\xcbI,IU\x84\x00\x00";
        let decoded = decode(encoded).unwrap();
        let string = String::from_utf8(decoded).unwrap();

        assert_eq!("Hello Deflate!!!!!!!", string);
    }
}
