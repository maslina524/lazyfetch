use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::huffman::stream::Stream;

#[derive(Debug, Clone)]
pub struct HuffmanNode {
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
    symb: Option<u32>,
}

impl HuffmanNode {
    pub const fn new() -> Self {
        Self {
            left: None,
            right: None,
            symb: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HuffmanTree {
    root: HuffmanNode,
}

impl HuffmanTree {
    pub const fn new() -> Self {
        Self {
            root: HuffmanNode::new(),
        }
    }

    pub fn from_alphabet_and_bl_list(alphabet: &[u32], bl_list: &[u8]) -> Self {
        assert_eq!(alphabet.len(), bl_list.len());

        let mut pairs: Vec<(u8, u32)> = alphabet
            .iter()
            .zip(bl_list.iter())
            .filter(|&(_, &len)| len > 0)
            .map(|(&symb, &len)| (len, symb))
            .collect();

        pairs.sort_by_key(|&(len, symb)| (len, symb));

        let max_bits = *bl_list.iter().max().unwrap_or(&0) as usize;

        let mut bl_count = vec![0; max_bits + 1];
        for &(len, _) in &pairs {
            bl_count[len as usize] += 1;
        }

        let mut next_code = vec![0; max_bits + 1];
        let mut code = 0;
        for bits in 1..=max_bits {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        let mut tree = Self::new();

        for (len, symb) in pairs {
            let code = next_code[len as usize];
            tree.insert(code, len as usize, symb);
            next_code[len as usize] += 1;
        }

        tree
    }

    pub fn insert(&mut self, code: u64, n: usize, symb: u32) {
        let mut curr = &mut self.root;

        for i in 0..n {
            let bit = (code >> (n - i - 1) & 1) as u8;
            let next = if bit == 1 {
                &mut curr.right
            } else {
                &mut curr.left
            };

            if next.is_none() {
                *next = Some(Box::new(HuffmanNode::new()));
            }

            curr = next.as_mut().unwrap();
        }

        curr.symb = Some(symb);
    }
}

pub fn decode_symb(stream: &mut Stream, tree: &HuffmanTree) -> Option<u32> {
    let mut curr = &tree.root;

    while curr.left.is_some() || curr.right.is_some() {
        let bit = stream.read_bit();
        let next = if bit == 1 {
            curr.right.as_ref()
        } else {
            curr.left.as_ref()
        };
        let node = next?;
        curr = node;
    }

    curr.symb
}
