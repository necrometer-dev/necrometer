//! Blocking HTTPS client over rustls. Replaces reqwest.
//!
//! What we need: GET a URL, get the status, body, and any `Link:`
//! header. That's it. No streaming, no POST, no cookies, no compression
//! (GitHub responses are small enough).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::ClientConfig;
use rustls::pki_types::ServerName;

use crate::error::{Error, Result};

pub struct Response {
    pub status: u16,
    pub body: String,
    /// `Link: <url>; rel="next"` — the next page URL, if any.
    pub next: Option<String>,
}

pub struct Client {
    cfg: Arc<ClientConfig>,
    user_agent: String,
}

impl Client {
    pub fn new() -> Result<Self> {
        let roots = webpki_roots::TLS_SERVER_ROOTS
            .iter()
            .cloned();
        let cfg = ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::from_iter(roots))
            .with_no_client_auth();
        Ok(Self {
            cfg: Arc::new(cfg),
            user_agent: "necrometer/0.2 (https://necrometer.dev)".into(),
        })
    }

    pub fn get(&self, url: &str, bearer: Option<&str>) -> Result<Response> {
        let (host, port, path) = parse_url(url)?;
        let server_name = ServerName::try_from(host.clone())
            .map_err(|e| Error::Tls(format!("dns name: {e}")))?;
        let conn = rustls::ClientConnection::new(self.cfg.clone(), server_name)
            .map_err(|e| Error::Tls(format!("handshake setup: {e}")))?;
        let mut sock = TcpStream::connect((host.as_str(), port))?;
        let mut conn = conn;
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        let mut req = format!(
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {}\r\nAccept: application/vnd.github+json\r\nConnection: close\r\n",
            self.user_agent
        );
        if let Some(tok) = bearer {
            req.push_str(&format!("Authorization: Bearer {tok}\r\n"));
        }
        req.push_str("\r\n");
        tls.write_all(req.as_bytes())?;
        let mut raw = Vec::new();
        tls.read_to_end(&mut raw)?;
        drop(tls);
        parse_response(&raw)
    }
}

fn parse_response(raw: &[u8]) -> Result<Response> {
    let split = find_subslice(raw, b"\r\n\r\n")
        .ok_or_else(|| Error::Http("no header terminator".into()))?;
    let head = &raw[..split];
    let body = &raw[split + 4..];
    let head_str = std::str::from_utf8(head).map_err(|_| Error::Http("bad utf-8 in head".into()))?;
    let mut lines = head_str.split("\r\n");
    let status_line = lines.next().ok_or_else(|| Error::Http("no status line".into()))?;
    let mut parts = status_line.split(' ');
    let _http = parts.next();
    let status: u16 = parts.next().ok_or_else(|| Error::Http("no status".into()))?
        .parse().map_err(|_| Error::Http("bad status".into()))?;
    let mut next: Option<String> = None;
    let mut is_chunked = false;
    let mut content_length: Option<usize> = None;
    for line in lines {
        if let Some(rest) = line.strip_prefix("Link:") {
            // <url>; rel="next"
            let s = rest.trim();
            if let Some(url) = s.split(';').next() {
                let url = url.trim().trim_matches('<').trim_matches('>');
                if s.contains("rel=\"next\"") || s.contains("rel=next") {
                    next = Some(url.to_string());
                }
            }
        } else if let Some(rest) = line.strip_prefix("Transfer-Encoding:") {
            if rest.trim().eq_ignore_ascii_case("chunked") { is_chunked = true; }
        } else if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let body = if is_chunked {
        dechunk(body)?
    } else if let Some(n) = content_length {
        body[..n.min(body.len())].to_vec()
    } else {
        body.to_vec()
    };
    Ok(Response {
        status,
        body: String::from_utf8_lossy(&body).into_owned(),
        next,
    })
}

fn dechunk(body: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line_end = find_subslice(&body[i..], b"\r\n")
            .ok_or_else(|| Error::Http("bad chunk".into()))?;
        let size_str = std::str::from_utf8(&body[i..i+line_end])
            .map_err(|_| Error::Http("bad utf-8 in chunk".into()))?;
        let size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|_| Error::Http("bad chunk size".into()))?;
        i += line_end + 2;
        if size == 0 { break; }
        if i + size > body.len() { return Err(Error::Http("chunk overruns body".into())); }
        out.extend_from_slice(&body[i..i+size]);
        i += size;
        if i + 2 <= body.len() { i += 2; } // trailing \r\n
    }
    Ok(out)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() { return None; }
    for i in 0..=hay.len() - needle.len() {
        if &hay[i..i+needle.len()] == needle { return Some(i); }
    }
    None
}

/// Split a URL into host / port / path. Handles `https://host[:port]/path?q`.
fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let rest = url.strip_prefix("https://")
        .ok_or_else(|| Error::Http("only https supported".into()))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.find(':') {
        Some(i) => {
            let p: u16 = authority[i+1..].parse()
                .map_err(|_| Error::Http("bad port".into()))?;
            (&authority[..i], p)
        }
        None => (authority, 443),
    };
    Ok((host.to_string(), port, path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_split() {
        let (h, p, pa) = parse_url("https://api.github.com/users/foo/repos?per_page=100").unwrap();
        assert_eq!(h, "api.github.com");
        assert_eq!(p, 443);
        assert_eq!(pa, "/users/foo/repos?per_page=100");
    }
}