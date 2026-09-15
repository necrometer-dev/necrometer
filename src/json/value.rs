//! `Value` type, serializer, and accessors.

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(i64),
    Str(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

pub fn write(out: &mut String, v: &Value) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Num(n) => out.push_str(&n.to_string()),
        Value::Str(s) => write_str(out, s),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 { out.push(','); }
                write(out, x);
            }
            out.push(']');
        }
        Value::Object(o) => {
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                if i > 0 { out.push(','); }
                write_str(out, k);
                out.push(':');
                write(out, x);
            }
            out.push('}');
        }
    }
}

fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Object(o) = self {
            for (k, v) in o {
                if k == key { return Some(v); }
            }
        }
        None
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Value::Str(s) = self { Some(s) } else { None }
    }
    pub fn as_i64(&self) -> Option<i64> {
        if let Value::Num(n) = self { Some(*n) } else { None }
    }
    pub fn as_u64(&self) -> Option<u64> {
        if let Value::Num(n) = self { Some((*n).max(0) as u64) } else { None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self { Some(*b) } else { None }
    }
}

#[allow(dead_code)]
pub fn _unused_error() -> Error { Error::Json("".into()) }

#[allow(dead_code)]
pub fn _unused_result() -> Result<()> { Ok(()) }
