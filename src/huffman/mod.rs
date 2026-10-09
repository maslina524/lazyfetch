#![doc = include_str!("README.md")]

mod stream;
mod tree;

pub use stream::Stream;
pub use tree::{HuffmanTree, decode_symb};
