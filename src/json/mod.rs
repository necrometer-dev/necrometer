//! Hand-rolled JSON parser + serializer. Replaces serde_json.
//!
//! Only what we need: parse JSON, serialize `Value`, and accessors.
//! Recursive-descent over `&[u8]`. No schema, no derive, no reflection.

mod parser;
mod value;

pub use value::Value;

use crate::error::{Error, Result};

pub fn parse(input: &str) -> Result<Value> {
    let mut p = parser::Parser::new(input.as_bytes());
    p.skip_ws();
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos < p.bytes.len() {
        return Err(Error::Json(format!("trailing at {}", p.pos)));
    }
    Ok(v)
}

pub fn serialize(v: &Value) -> String {
    let mut out = String::with_capacity(64);
    value::write(&mut out, v);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_repo() {
        let raw = r#"[{"name":"x","pushed_at":"2024-01-15T12:34:56Z","created_at":"2023-01-15T12:34:56Z","archived":false,"fork":false,"stargazers_count":42,"html_url":"https://example.com/x"}]"#;
        let v = parse(raw).unwrap();
        let arr = match v { Value::Array(a) => a, _ => panic!() };
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].get("name").unwrap().as_str(), Some("x"));
        assert_eq!(arr[0].get("stargazers_count").unwrap().as_i64(), Some(42));
        let s = serialize(&Value::Array(arr));
        assert!(s.contains("\"name\":\"x\""));
    }

    #[test]
    fn rejects_trailing() {
        assert!(parse("{\"a\":1} garbage").is_err());
        assert!(parse("nope").is_err());
    }
}