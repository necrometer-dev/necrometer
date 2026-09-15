//! Blocking HTTPS client over rustls. Replaces reqwest.
//!
//! What we need: GET a URL, get the status, body, and any `Link:`
//! header. That's it. No streaming, no POST, no cookies, no compression
//! (GitHub responses are small enough).

mod parse;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::ClientConfig;

use crate::error::{Error, Result};
use parse::{find_subslice, parse_response, parse_url, response_complete};

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
        let roots = webpki_roots::TLS_SERVER_ROOTS.iter().cloned();
        let cfg = ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::from_iter(roots))
            .with_no_client_auth();
        Ok(Self {
            cfg: Arc::new(cfg),
            user_agent: "necrometer/0.4 (https://necrometer.dev)".into(),
        })
    }

    pub fn get(&self, url: &str, bearer: Option<&str>) -> Result<Response> {
        let (host, port, path) = parse_url(url)?;
        let server_name =
            ServerName::try_from(host.clone()).map_err(|e| Error::Tls(format!("dns name: {e}")))?;
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
        parse_response(&read_http(&mut tls)?)
    }
}

/// GitHub (and many CDNs) FIN the TCP socket without a TLS
/// `close_notify`. `read_to_end` then fails even when the HTTP
/// body is complete. Stop as soon as we have headers + body.
fn read_http(r: &mut impl Read) -> Result<Vec<u8>> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&buf[..n]);
                if response_complete(&raw) {
                    break;
                }
            }
            Err(e) if is_tls_hangup(&e) => {
                if find_subslice(&raw, b"\r\n\r\n").is_some() {
                    break;
                }
                return Err(e.into());
            }
            Err(e) => return Err(e.into()),
        }
    }
    if find_subslice(&raw, b"\r\n\r\n").is_none() {
        return Err(Error::Http("truncated response".into()));
    }
    Ok(raw)
}

fn is_tls_hangup(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::UnexpectedEof || e.to_string().contains("close_notify")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangup_is_eof() {
        let e = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "close_notify");
        assert!(is_tls_hangup(&e));
        let e = std::io::Error::new(std::io::ErrorKind::WouldBlock, "nope");
        assert!(!is_tls_hangup(&e));
    }

    #[test]
    fn read_http_stops_at_content_length() {
        let payload = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}TRAILING";
        let mut cur = std::io::Cursor::new(&payload[..]);
        let raw = read_http(&mut cur).unwrap();
        let r = parse_response(&raw).unwrap();
        assert_eq!(r.body, "{}");
    }
}
