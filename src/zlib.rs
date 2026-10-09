use core::{cmp, iter::repeat_n};

use alloc::vec::Vec;

use crate::{
    abort,
    huffman::{HuffmanTree, Stream, decode_symb},
};

const LENGTH_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA_BITS: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA_BITS: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CODE_LEN_CODES_ORDER: [u32; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn extend_at<T: Clone>(dst: &mut [T], offset: usize, src: impl IntoIterator<Item = T>) {
    for (slot, val) in dst[offset..].iter_mut().zip(src) {
        *slot = val;
    }
}

pub fn inflate_block_no_compression(stream: &mut Stream, inflated: &mut Vec<u8>) {
    stream.align_to_byte();

    let len = stream.read_bytes(2);
    let _nlen = stream.read_bytes(2);

    for _ in 0..len {
        inflated.push(stream.read_byte());
    }
}

pub fn inflate_block_fixed_huffman(stream: &mut Stream, inflated: &mut Vec<u8>) {
    let mut bl_list = [0u8; 288];
    extend_at(&mut bl_list, 0, repeat_n(8, 144));
    extend_at(&mut bl_list, 144, repeat_n(9, 256 - 144));
    extend_at(&mut bl_list, 256, repeat_n(7, 280 - 256));
    extend_at(&mut bl_list, 280, repeat_n(8, 288 - 280));

    let mut alphabet = [0u32; 288];
    extend_at(&mut alphabet, 0, 0..288);

    let literal_len_tree = HuffmanTree::from_alphabet_and_bl_list(&alphabet, &bl_list);

    let bl_list = [5; 30];

    let mut alphabet = [0u32; 30];
    extend_at(&mut alphabet, 0, 0..30);

    let distance_tree = HuffmanTree::from_alphabet_and_bl_list(&alphabet, &bl_list);

    inflate_compressed_block(stream, &literal_len_tree, &distance_tree, inflated);
}

pub fn inflate_compressed_block(
    stream: &mut Stream,
    literal_len_tree: &HuffmanTree,
    distance_tree: &HuffmanTree,
    inflated: &mut Vec<u8>,
) {
    loop {
        let symb = decode_symb(stream, literal_len_tree).unwrap();

        if symb < 256 {
            inflated.push(symb as u8);
        } else if symb == 256 {
            return;
        } else if (257..=285).contains(&symb) {
            let i = (symb - 257) as usize;
            let len = LENGTH_BASE[i] + stream.read_bits(LENGTH_EXTRA_BITS[i] as usize) as u32;
            let dist_symb = decode_symb(stream, distance_tree).unwrap() as usize;
            let distance = DISTANCE_BASE[dist_symb]
                + stream.read_bits(DISTANCE_EXTRA_BITS[dist_symb] as usize) as u32;

            for _ in 0..len {
                inflated.push(inflated[inflated.len() - distance as usize]);
            }
        } else {
            abort!("Invalid Huffman symbol: {symb}");
        }
    }
}

pub fn decode_trees(stream: &mut Stream) -> (HuffmanTree, HuffmanTree) {
    let hlit = stream.read_bits(5) + 257;
    let hdist = stream.read_bits(5) + 1;
    let hclen = stream.read_bits(4) + 4;

    let mut code_len_bl_list = [0; 19];
    for i in 0..hclen {
        code_len_bl_list[CODE_LEN_CODES_ORDER[i as usize] as usize] = stream.read_bits(3) as u8;
    }

    let mut code_len_alphabet = [0u32; 19];
    extend_at(&mut code_len_alphabet, 0, 0..19);

    let code_len_tree =
        HuffmanTree::from_alphabet_and_bl_list(&code_len_alphabet, &code_len_bl_list);

    let mut bl_list = Vec::with_capacity((hlit + hdist) as usize);
    while bl_list.len() < (hlit + hdist) as usize {
        let symb = decode_symb(stream, &code_len_tree).unwrap();
        if symb < 16 {
            bl_list.push(symb as u8);
        } else if symb == 16 {
            let prev_code_len = bl_list[bl_list.len() - 1];
            let repeat_len = stream.read_bits(2) + 3;
            for _ in 0..repeat_len {
                bl_list.push(prev_code_len);
            }
        } else if symb == 17 {
            let repeat_len = stream.read_bits(3) + 3;
            bl_list.extend(repeat_n(0, repeat_len as usize));
        } else {
            let repeat_len = stream.read_bits(7) + 11;
            bl_list.extend(repeat_n(0, repeat_len as usize));
        }
    }

    let literal_count = cmp::min(hlit as usize, 288);
    let literal_alphabet: Vec<u32> = (0..literal_count as u32).collect();
    let literal_len_tree =
        HuffmanTree::from_alphabet_and_bl_list(&literal_alphabet, &bl_list[..literal_count]);

    let dist_count = cmp::min(hdist as usize, 30);
    let dist_alphabet: Vec<u32> = (0..dist_count as u32).collect();
    let distance_tree = HuffmanTree::from_alphabet_and_bl_list(
        &dist_alphabet,
        &bl_list[hlit as usize..hlit as usize + dist_count],
    );

    (literal_len_tree, distance_tree)
}

pub fn inflate_block_dynamic_huffman(stream: &mut Stream, inflated: &mut Vec<u8>) {
    let (literal_len_tree, distance_tree) = decode_trees(stream);
    inflate_compressed_block(stream, &literal_len_tree, &distance_tree, inflated);
}

fn inflate(stream: &mut Stream, buf: &mut Vec<u8>) {
    let mut bfinal = 0;
    while bfinal == 0 {
        bfinal = stream.read_bit();
        let btype = stream.read_bits(2);

        match btype {
            0b00 => inflate_block_no_compression(stream, buf),
            0b01 => inflate_block_fixed_huffman(stream, buf),
            0b10 => inflate_block_dynamic_huffman(stream, buf),
            _ => {}
        }
    }
}

pub type Adler32 = u32;

pub fn decompress(bytes: &'_ [u8], buf: &mut Vec<u8>) -> Adler32 {
    let mut stream = Stream::new(bytes);
    stream.read_byte();
    stream.read_byte();
    inflate(&mut stream, buf);
    stream.read_bytes(4) as u32
}

#[cfg(test)]
mod tests {
    use crate::zlib::decompress;

    #[test]
    fn decompress_logo_test() {
        let compressed = include_bytes!(concat!(env!("LOGO_OUT_DIR"), "/temp/d/debian.txt"));
        let mut decompressed = Vec::new();

        decompress(compressed, &mut decompressed);
        let string = String::from_utf8(decompressed).expect("Non Utf8 logo");

        println!("{string}");
    }
}
