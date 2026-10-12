#![doc = include_str!("README.md")]

mod lexer;
pub mod parser;

use lexer::TokenStream;
use parser::Parser;
pub use parser::{Map, Value};

use alloc::string::{String, ToString};

use crate::{imp::fs, imp::path::Path};

#[derive(Debug)]
pub struct Json;

impl Json {
    pub fn from_file(path: impl Into<Path>) -> Result<Map, String> {
        let string = match fs::read_to_string(path.into()) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };
        let map = Self::from_str(&string)?;
        Ok(map)
    }

    pub fn from_str(s: &str) -> Result<Map, String> {
        let stream = TokenStream::new(s);
        let mut parser = Parser::new(stream);
        parser.parse_object()
    }
}

#[cfg(test)]
mod tests {
    use crate::json::Json;

    #[test]
    fn single_chars_test() {
        let source = r#"{
            "key": "value",
            "array": [80, 42.55, true, null, "string"],
            "dict": {
                "type": "title",
                "idx": 1
            },
            "nullable": null,
            "boolean": true
        }"#;

        let json = Json::from_str(source);
        println!("{json:#?}");
    }
}
