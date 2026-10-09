#![doc = include_str!("README.md")]

pub mod concat;
pub use concat::ConcatStr;

pub mod smol;
pub use smol::SmolStr;
