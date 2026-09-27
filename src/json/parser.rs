use core::str::FromStr;

use alloc::{
    string::String,
    vec::Vec,
};

use crate::{
    json::lexer::{Token, TokenStream},
    format
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Self>),
    Dict(Map),
    Null
}

impl Value {
    pub const fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None
        }
    }

    pub const fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(s) => Some(*s),
            _ => None
        }
    }

    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(s) => Some(*s),
            _ => None
        }
    }

    pub const fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(s) => Some(s),
            _ => None
        }
    }

    pub const fn as_object(&self) -> Option<&Map> {
        match self {
            Self::Dict(s) => Some(s),
            _ => None
        }
    }

    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Dict(_))
    }

    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Map(Vec<(String, Value)>);

impl Map {
    pub const fn new() -> Self { 
        Self(Vec::new()) 
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.0.push((key, value));
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_string(&self, key: &str) -> Option<&String> {
        self.get(key).and_then(Value::as_string)
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(Value::as_number)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(Value::as_bool)
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key).and_then(Value::as_array)
    }

    pub fn get_object(&self, key: &str) -> Option<&Self> {
        self.get(key).and_then(Value::as_object)
    }
}

pub struct Parser {
    iter: TokenStream,
    current: Option<Token>,
}

impl Parser {
    pub fn new(mut iter: TokenStream) -> Self {
        let current = iter.next();
        Self { iter, current }
    }

    const fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.current.take();
        self.current = self.iter.next();
        token
    }

    fn consume(&mut self, expected: &Token) -> Result<Token, String> {
        let token = self.next().ok_or("Unexpected end of input")?;
        if token == *expected {
            Ok(token)
        } else {
            Err(format!("Expected {expected:?}, got {token:?}"))
        }
    }

    pub fn parse_value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some(Token::LCurly) => {
                Ok(Value::Dict(self.parse_object()?))
            }
            Some(Token::LBrace) => {
                Ok(Value::Array(self.parse_array()?))
            }
            Some(Token::String(s)) => {
                // The lexer already strips the surrounding quotes and
                // resolves escape sequences (including \uXXXX), so the
                // token's content is the final string value as-is.
                let s = s.clone();
                self.next();
                Ok(Value::String(s))
            }
            Some(Token::Number(s)) => {
                let s = s.clone();
                self.next();
                let num = f64::from_str(&s)
                    .map_err(|_| "Invalid number format")?;
                Ok(Value::Number(num))
            }
            Some(Token::True) => {
                self.next();
                Ok(Value::Bool(true))
            }
            Some(Token::False) => {
                self.next();
                Ok(Value::Bool(false))
            }
            Some(Token::Null) => {
                self.next();
                Ok(Value::Null)
            }
            _ => Err("Unexpected token while parsing value".into()),
        }
    }

    pub fn parse_object(&mut self) -> Result<Map, String> {
        self.next(); // consume '{'
        let mut map = Map::new();

        if matches!(self.peek(), Some(Token::RCurly)) {
            self.next();
            return Ok(map);
        }

        loop {
            let key_token = self.next().ok_or("Expected object key")?;
            let Token::String(key) = key_token else { 
                return Err("Object key must be a string".into()) 
            };

            self.consume(&Token::Colon)?;

            let value = self.parse_value()?;
            map.insert(key, value);

            match self.peek() {
                Some(Token::Comma) => {
                    self.next(); // consume ','
                    if matches!(self.peek(), Some(Token::RCurly)) {
                        self.next(); // consume '}'
                        break;
                    }
                }
                Some(Token::RCurly) => {
                    self.next();
                    break;
                }
                _ => return Err("Expected ',' or '}}' in object".into()),
            }
        }

        Ok(map)
    }

    pub fn parse_array(&mut self) -> Result<Vec<Value>, String> {
        self.next(); // consume '['
        let mut vec = Vec::new();

        if matches!(self.peek(), Some(Token::RBrace)) {
            self.next();
            return Ok(vec);
        }

        loop {
            let value = self.parse_value()?;
            vec.push(value);

            match self.peek() {
                Some(Token::Comma) => {
                    self.next(); // consume ','
                    if matches!(self.peek(), Some(Token::RBrace)) {
                        self.next(); // consume ']'
                        break;
                    }
                }
                Some(Token::RBrace) => {
                    self.next();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".into()),
            }
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::json::lexer::TokenStream;
    use crate::json::parser::Parser;

    #[test]
    fn single_chars_test() {
        let source = r#"{
            "key": "value",
            "array": [80, 42.55, true, null, "string"],
            "dict": {
                "type": "title",
                "idx": 1
            }
        }"#;
        let stream = TokenStream::new(source);

        let mut parser = Parser::new(stream);
        let obj = parser.parse_object();

        println!("{obj:#?}");
    }
}