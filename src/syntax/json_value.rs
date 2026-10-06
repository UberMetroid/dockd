//! Minimal JSON Value representation and accessors in pure standard library.
//!
//! Provides ergonomic inspection methods for structured protocol payloads.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_f64().map(|n| n.round() as i64)
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.as_f64()
            .filter(|&n| n >= 0.0)
            .map(|n| n.round() as u64)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(m) => Some(m),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(m) => m.get(key),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(JsonValue::as_str)
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(JsonValue::as_f64)
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(JsonValue::as_i64)
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(JsonValue::as_u64)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(JsonValue::as_bool)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}
