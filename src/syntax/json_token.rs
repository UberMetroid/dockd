//! Lexical tokenization for minimal JSON parsing without external dependencies.
//!
//! Emits discrete lexical tokens while preserving precision and handling string
//! escape sequences according to RFC 8259.

#[derive(Debug, Clone, PartialEq)]
pub enum JsonToken {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
}

pub struct JsonLexer<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
}

impl<'a> JsonLexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let peeked = chars.next();
        Self { chars, peeked }
    }

    fn advance(&mut self) -> Option<char> {
        let current = self.peeked;
        self.peeked = self.chars.next();
        current
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peeked {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Option<JsonToken>, String> {
        self.skip_whitespace();
        let ch = match self.peeked {
            Some(c) => c,
            None => return Ok(None),
        };

        match ch {
            '{' => {
                self.advance();
                Ok(Some(JsonToken::OpenBrace))
            }
            '}' => {
                self.advance();
                Ok(Some(JsonToken::CloseBrace))
            }
            '[' => {
                self.advance();
                Ok(Some(JsonToken::OpenBracket))
            }
            ']' => {
                self.advance();
                Ok(Some(JsonToken::CloseBracket))
            }
            ':' => {
                self.advance();
                Ok(Some(JsonToken::Colon))
            }
            ',' => {
                self.advance();
                Ok(Some(JsonToken::Comma))
            }
            '"' => self.lex_string().map(|s| Some(JsonToken::String(s))),
            't' | 'f' => self.lex_bool().map(|b| Some(JsonToken::Bool(b))),
            'n' => self.lex_null().map(|_| Some(JsonToken::Null)),
            '-' | '0'..='9' => self.lex_number().map(|n| Some(JsonToken::Number(n))),
            other => Err(format!("Unexpected character in JSON: {other}")),
        }
    }

    fn lex_string(&mut self) -> Result<String, String> {
        self.advance(); // consume opening quote
        let mut out = String::new();
        while let Some(ch) = self.advance() {
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let esc = self
                        .advance()
                        .ok_or_else(|| "Unterminated string escape".to_string())?;
                    match esc {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\x08'),
                        'f' => out.push('\x0C'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                let h = self
                                    .advance()
                                    .ok_or_else(|| "Truncated unicode escape".to_string())?;
                                hex.push(h);
                            }
                            let cp = u32::from_str_radix(&hex, 16)
                                .map_err(|e| format!("Invalid unicode escape: {e}"))?;
                            let ch = char::from_u32(cp)
                                .ok_or_else(|| "Invalid unicode scalar value".to_string())?;
                            out.push(ch);
                        }
                        other => return Err(format!("Unknown escape sequence: \\{other}")),
                    }
                }
                c => out.push(c),
            }
        }
        Err("Unterminated string literal in JSON".to_string())
    }

    fn lex_bool(&mut self) -> Result<bool, String> {
        let mut word = String::new();
        while let Some(ch) = self.peeked {
            if ch.is_ascii_alphabetic() {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match word.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(format!("Expected boolean, found '{other}'")),
        }
    }

    fn lex_null(&mut self) -> Result<(), String> {
        let mut word = String::new();
        while let Some(ch) = self.peeked {
            if ch.is_ascii_alphabetic() {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if word == "null" {
            Ok(())
        } else {
            Err(format!("Expected null, found '{word}'"))
        }
    }

    fn lex_number(&mut self) -> Result<f64, String> {
        let mut num_str = String::new();
        while let Some(ch) = self.peeked {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E'
            {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str
            .parse::<f64>()
            .map_err(|e| format!("Malformed number '{num_str}': {e}"))
    }
}
