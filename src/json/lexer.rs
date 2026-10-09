use alloc::{string::String, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    // Literals
    String(String), // with "..."
    Number(String),
    False,
    True,
    Null,

    // Paren
    LCurly,
    RCurly,
    LBrace,
    RBrace,

    Comma,
    Colon,
}

#[derive(Clone)]
pub struct TokenStream {
    chars: Vec<char>,
    pos: usize,
}

impl TokenStream {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
            self.pos += 1;
        }
    }

    fn read_single(&self) -> Option<Token> {
        match self.chars[self.pos] {
            '{' => Some(Token::LCurly),
            '}' => Some(Token::RCurly),
            '[' => Some(Token::LBrace),
            ']' => Some(Token::RBrace),
            ',' => Some(Token::Comma),
            ':' => Some(Token::Colon),
            _ => None,
        }
    }

    fn read_string(&mut self) -> Token {
        // skip opening quote
        self.pos += 1;
        let mut ret = String::with_capacity(24);

        while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
            let ch = self.chars[self.pos];

            if ch == '\\' && self.pos + 1 < self.chars.len() {
                self.pos += 1;
                match self.chars[self.pos] {
                    '"' => ret.push('"'),
                    '\\' => ret.push('\\'),
                    '/' => ret.push('/'),
                    'n' => ret.push('\n'),
                    't' => ret.push('\t'),
                    'r' => ret.push('\r'),
                    'b' => ret.push('\u{0008}'),
                    'f' => ret.push('\u{000C}'),
                    'u' => {
                        if self.pos + 4 < self.chars.len() {
                            let hex: String =
                                self.chars[self.pos + 1..self.pos + 5].iter().collect();

                            if let Ok(code) = u32::from_str_radix(&hex, 16)
                                && let Some(c) = char::from_u32(code)
                            {
                                ret.push(c);
                            }
                            self.pos += 4;
                        }
                    }
                    other => ret.push(other),
                }
            } else {
                ret.push(ch);
            }
            self.pos += 1;
        }

        if self.pos < self.chars.len() {
            self.pos += 1;
        }

        Token::String(ret)
    }

    fn read_number(&mut self) -> Token {
        let mut ret = String::with_capacity(8);

        loop {
            let ch = self.chars[self.pos];
            if self.pos >= self.chars.len() || (ch != '-' && ch != '.' && !ch.is_numeric()) {
                break;
            }

            ret.push(self.chars[self.pos]);
            self.pos += 1;
        }

        Token::Number(ret)
    }

    fn read_keyword(&mut self, string: &str, token: Token) -> Option<Token> {
        let len = string.len();

        if self.pos + len <= self.chars.len()
            && self.chars[self.pos..self.pos + len]
                .iter()
                .copied()
                .eq(string.chars())
        {
            self.pos += len;
            return Some(token);
        }

        None
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.chars.len() {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                return None;
            }

            let ch = self.chars[self.pos];
            if self.pos + 1 < self.chars.len() && ch == '/' && self.chars[self.pos + 1] == '/' {
                self.skip_comment();
                continue;
            }

            // Single
            if let Some(t) = self.read_single() {
                self.pos += 1;
                return Some(t);
            }

            // Strings
            if ch == '"' {
                return Some(self.read_string());
            }

            // Number
            if ch == '-' || ch.is_numeric() {
                return Some(self.read_number());
            }

            // Keywords
            if let Some(token) = self.read_keyword("null", Token::Null) {
                return Some(token);
            }
            if let Some(token) = self.read_keyword("true", Token::True) {
                return Some(token);
            }
            if let Some(token) = self.read_keyword("false", Token::False) {
                return Some(token);
            }

            self.pos += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::json::lexer::{Token, TokenStream};

    #[test]
    fn single_chars_test() {
        let source = "{}: [,]";
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.next(), Some(Token::LCurly));
        assert_eq!(stream.next(), Some(Token::RCurly));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::LBrace));
        assert_eq!(stream.next(), Some(Token::Comma));
        assert_eq!(stream.next(), Some(Token::RBrace));
    }

    #[test]
    fn string_test() {
        let source = "{\"key\": \"value\"}";
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.next(), Some(Token::LCurly));
        assert_eq!(stream.next(), Some(Token::String("\"key\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::String("\"value\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::RCurly));
    }

    #[test]
    fn keywods_test() {
        let source = "{\"boolean\": true, \"null\": null}";
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.next(), Some(Token::LCurly));
        assert_eq!(stream.next(), Some(Token::String("\"boolean\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::True));
        assert_eq!(stream.next(), Some(Token::Comma));
        assert_eq!(stream.next(), Some(Token::String("\"null\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::Null));
        assert_eq!(stream.next(), Some(Token::RCurly));
    }

    #[test]
    fn number_test() {
        let source = r#"{
            "array": [
                42,
                55,
                10.70,
                -98.84
            ]
        }"#;
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.next(), Some(Token::LCurly));
        assert_eq!(stream.next(), Some(Token::String("\"array\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::LBrace));

        assert_eq!(stream.next(), Some(Token::Number("42".to_owned())));
        assert_eq!(stream.next(), Some(Token::Comma));

        assert_eq!(stream.next(), Some(Token::Number("55".to_owned())));
        assert_eq!(stream.next(), Some(Token::Comma));

        assert_eq!(stream.next(), Some(Token::Number("10.70".to_owned())));
        assert_eq!(stream.next(), Some(Token::Comma));

        assert_eq!(stream.next(), Some(Token::Number("-98.84".to_owned())));

        assert_eq!(stream.next(), Some(Token::RBrace));
        assert_eq!(stream.next(), Some(Token::RCurly));
    }

    #[test]
    fn comments_test() {
        let source = r#"{
            // Comment
            "key": "value"
            // Another comment
        }"#;
        let mut stream = TokenStream::new(source);

        assert_eq!(stream.next(), Some(Token::LCurly));
        assert_eq!(stream.next(), Some(Token::String("\"key\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::Colon));
        assert_eq!(stream.next(), Some(Token::String("\"value\"".to_owned())));
        assert_eq!(stream.next(), Some(Token::RCurly));
    }

    #[test]
    fn string_extra_test() {
        let source = r#"
        "Cow says: \"Hello World!\""
        "#;
        let mut stream = TokenStream::new(source);

        assert_eq!(
            stream.next(),
            Some(Token::String("Cow says: \"Hello World!\"".to_owned()))
        );
    }
}
