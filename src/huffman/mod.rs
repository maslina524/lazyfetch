#![doc = include_str!("README.md")]

mod tree;
mod stream;

pub use tree::{HuffmanTree, decode_symb};
pub use stream::Stream;