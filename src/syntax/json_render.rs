//! JSON serialization for JsonValue without third-party crates.
//!
//! Handles proper string escaping and structured formatting for protocol payloads.

use super::json_value::JsonValue;

pub fn render(value: &JsonValue) -> String {
    let mut out = String::new();
    write_value(value, &mut out);
    out
}

pub fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn write_value(value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{n}"));
            }
        }
        JsonValue::String(s) => out.push_str(&escape_str(s)),
        JsonValue::Array(arr) => {
            out.push('[');
            for (i, elem) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(elem, out);
            }
            out.push(']');
        }
        JsonValue::Object(map) => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&escape_str(k));
                out.push(':');
                write_value(v, out);
            }
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::json_parse::parse;

    #[test]
    fn render_and_parse_roundtrip() {
        let input = r#"{"count":42,"name":"dockd","running":true,"tags":["system","dock"]}"#;
        let val = parse(input).unwrap();
        let rendered = render(&val);
        let roundtrip = parse(&rendered).unwrap();
        assert_eq!(val, roundtrip);
    }

    #[test]
    fn escape_special_characters() {
        let escaped = escape_str("foo\n\"bar\"\t\\baz");
        assert_eq!(escaped, r#""foo\n\"bar\"\t\\baz""#);
    }
}
