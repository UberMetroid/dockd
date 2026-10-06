//! Recursive descent parser for JSON text into JsonValue.
//!
//! Enforces well-formed structures, bounded nesting depth, and clean error returns.

use super::json_token::{JsonLexer, JsonToken};
use super::json_value::JsonValue;
use std::collections::BTreeMap;

const MAX_DEPTH: usize = 64;

pub fn parse(input: &str) -> Result<JsonValue, String> {
    let mut lexer = JsonLexer::new(input);
    let first = lexer.next_token()?;
    let value = parse_value(first, &mut lexer, 0)?;
    if let Some(trailing) = lexer.next_token()? {
        return Err(format!("Trailing token after JSON payload: {trailing:?}"));
    }
    Ok(value)
}

fn parse_value(
    current: Option<JsonToken>,
    lexer: &mut JsonLexer<'_>,
    depth: usize,
) -> Result<JsonValue, String> {
    if depth > MAX_DEPTH {
        return Err("Exceeded maximum recursion depth parsing JSON".to_string());
    }

    match current {
        Some(JsonToken::Null) => Ok(JsonValue::Null),
        Some(JsonToken::Bool(b)) => Ok(JsonValue::Bool(b)),
        Some(JsonToken::Number(n)) => Ok(JsonValue::Number(n)),
        Some(JsonToken::String(s)) => Ok(JsonValue::String(s)),
        Some(JsonToken::OpenBracket) => parse_array(lexer, depth + 1),
        Some(JsonToken::OpenBrace) => parse_object(lexer, depth + 1),
        Some(token) => Err(format!("Unexpected token in value position: {token:?}")),
        None => Err("Unexpected end of JSON input".to_string()),
    }
}

fn parse_array(lexer: &mut JsonLexer<'_>, depth: usize) -> Result<JsonValue, String> {
    let mut elements = Vec::new();
    let mut first = true;

    loop {
        let token = lexer.next_token()?;
        match token {
            Some(JsonToken::CloseBracket) => return Ok(JsonValue::Array(elements)),
            Some(JsonToken::Comma) if !first => {
                let next = lexer.next_token()?;
                elements.push(parse_value(next, lexer, depth)?);
            }
            Some(t) if first => {
                first = false;
                elements.push(parse_value(Some(t), lexer, depth)?);
            }
            Some(other) => return Err(format!("Unexpected token in array: {other:?}")),
            None => return Err("Unterminated array in JSON".to_string()),
        }
    }
}

fn parse_object(lexer: &mut JsonLexer<'_>, depth: usize) -> Result<JsonValue, String> {
    let mut map = BTreeMap::new();
    let mut first = true;

    loop {
        let token = lexer.next_token()?;
        match token {
            Some(JsonToken::CloseBrace) => return Ok(JsonValue::Object(map)),
            Some(JsonToken::Comma) if !first => {
                let (k, v) = parse_key_value(lexer, depth)?;
                map.insert(k, v);
            }
            Some(JsonToken::String(k)) if first => {
                first = false;
                let colon = lexer.next_token()?;
                if colon != Some(JsonToken::Colon) {
                    return Err("Expected colon after object key".to_string());
                }
                let val_tok = lexer.next_token()?;
                let val = parse_value(val_tok, lexer, depth)?;
                map.insert(k, val);
            }
            Some(other) => return Err(format!("Unexpected token in object: {other:?}")),
            None => return Err("Unterminated object in JSON".to_string()),
        }
    }
}

fn parse_key_value(lexer: &mut JsonLexer<'_>, depth: usize) -> Result<(String, JsonValue), String> {
    let key_tok = lexer.next_token()?;
    let key = match key_tok {
        Some(JsonToken::String(k)) => k,
        other => return Err(format!("Expected string key in object, found {other:?}")),
    };
    let colon = lexer.next_token()?;
    if colon != Some(JsonToken::Colon) {
        return Err("Expected colon after object key".to_string());
    }
    let val_tok = lexer.next_token()?;
    let val = parse_value(val_tok, lexer, depth)?;
    Ok((key, val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_primitives() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
        assert_eq!(parse("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse("-10.5").unwrap(), JsonValue::Number(-10.5));
    }

    #[test]
    fn parse_escaped_string() {
        let input = r#""Hello \"world\"\n\t\u0041""#;
        assert_eq!(
            parse(input).unwrap(),
            JsonValue::String("Hello \"world\"\n\tA".to_string())
        );
    }

    #[test]
    fn parse_complex_object() {
        let json = r#"{"name": "firefox", "pinned": true, "count": 2, "tags": ["browser", "web"]}"#;
        let val = parse(json).unwrap();
        assert_eq!(val.get_str("name"), Some("firefox"));
        assert_eq!(val.get_bool("pinned"), Some(true));
        assert_eq!(val.get_i64("count"), Some(2));
        assert!(val.get("tags").and_then(JsonValue::as_array).is_some());
    }

    #[test]
    fn reject_malformed() {
        assert!(parse("{unterminated").is_err());
        assert!(parse("[1, 2,").is_err());
    }
}
