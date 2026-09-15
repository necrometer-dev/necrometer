//! Recursive-descent parser. State: byte slice + cursor.

use crate::error::Error;
use super::value::Value;

pub struct Parser<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self { Self { bytes, pos: 0 } }

    pub fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        self.skip_ws();
        if self.pos >= self.bytes.len() {
            return Err(Error::Json("unexpected end".into()));
        }
        match self.bytes[self.pos] {
            b'n' => { self.expect("null")?; Ok(Value::Null) }
            b't' => { self.expect("true")?; Ok(Value::Bool(true)) }
            b'f' => { self.expect("false")?; Ok(Value::Bool(false)) }
            b'"' => Ok(Value::Str(self.parse_string()?)),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(Error::Json(format!("bad token at {}", self.pos))),
        }
    }

    fn expect(&mut self, kw: &str) -> Result<(), Error> {
        let kb = kw.as_bytes();
        if self.bytes[self.pos..].starts_with(kb) {
            self.pos += kb.len();
            Ok(())
        } else {
            Err(Error::Json(format!("expected {kw}")))
        }
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        if self.bytes[self.pos] != b'"' {
            return Err(Error::Json("expected '\"'".into()));
        }
        self.pos += 1;
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let c = self.bytes[self.pos];
            match c {
                b'"' => { self.pos += 1; return Ok(out); }
                b'\\' => {
                    self.pos += 1;
                    if self.pos >= self.bytes.len() { return Err(Error::Json("eof in escape".into())); }
                    match self.bytes[self.pos] {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' => out.push('\u{08}'),
                        b'f' => out.push('\u{0C}'),
                        b'u' => {
                            self.pos += 1;
                            let cp = self.parse_hex4()?;
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                            continue;
                        }
                        _ => return Err(Error::Json("bad escape".into())),
                    }
                    self.pos += 1;
                }
                _ => {
                    let s = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| Error::Json("bad utf-8".into()))?;
                    let ch = s.chars().next().unwrap();
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
        Err(Error::Json("unterminated string".into()))
    }

    fn parse_hex4(&mut self) -> Result<u32, Error> {
        if self.pos + 4 > self.bytes.len() { return Err(Error::Json("short hex".into())); }
        let mut v: u32 = 0;
        for _ in 0..4 {
            let c = self.bytes[self.pos];
            let d = match c {
                b'0'..=b'9' => (c - b'0') as u32,
                b'a'..=b'f' => (c - b'a' + 10) as u32,
                b'A'..=b'F' => (c - b'A' + 10) as u32,
                _ => return Err(Error::Json("bad hex".into())),
            };
            v = (v << 4) | d;
            self.pos += 1;
        }
        Ok(v)
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let start = self.pos;
        if self.bytes[self.pos] == b'-' { self.pos += 1; }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| Error::Json("bad utf-8 in number".into()))?;
        let n: i64 = s.parse().map_err(|_| Error::Json(format!("bad number {s}")))?;
        Ok(Value::Num(n))
    }

    fn parse_array(&mut self) -> Result<Value, Error> {
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_ws();
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b']' {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            if self.pos >= self.bytes.len() { return Err(Error::Json("eof in array".into())); }
            match self.bytes[self.pos] {
                b',' => { self.pos += 1; }
                b']' => { self.pos += 1; return Ok(Value::Array(items)); }
                _ => return Err(Error::Json("bad array sep".into())),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value, Error> {
        self.pos += 1;
        let mut kvs = Vec::new();
        self.skip_ws();
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'}' {
            self.pos += 1;
            return Ok(Value::Object(kvs));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.pos >= self.bytes.len() || self.bytes[self.pos] != b':' {
                return Err(Error::Json("expected ':'".into()));
            }
            self.pos += 1;
            let val = self.parse_value()?;
            kvs.push((key, val));
            self.skip_ws();
            if self.pos >= self.bytes.len() { return Err(Error::Json("eof in object".into())); }
            match self.bytes[self.pos] {
                b',' => { self.pos += 1; }
                b'}' => { self.pos += 1; return Ok(Value::Object(kvs)); }
                _ => return Err(Error::Json("bad object sep".into())),
            }
        }
    }
}
