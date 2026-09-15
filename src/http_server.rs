//! Hand-rolled blocking HTTP/1.1 server. Replaces axum.
//!
//! Per-request: read request line + headers, route, write response.
//! No keep-alive (one request per connection), no streaming, no
//! websocket. Each connection runs in a `std::thread`.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use crate::error::Result;

pub type Handler = Arc<dyn Fn(Request) -> Response + Send + Sync + 'static>;

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub peer: String,
}

pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn html(s: impl Into<String>) -> Self {
        Response::with_type(200, "text/html; charset=utf-8", s.into().into_bytes())
    }
    pub fn svg(s: impl Into<String>) -> Self {
        Response::with_type(200, "image/svg+xml; charset=utf-8", s.into().into_bytes())
    }
    pub fn plain(s: impl Into<String>) -> Self {
        Response::with_type(200, "text/plain; charset=utf-8", s.into().into_bytes())
    }
    pub fn redirect(path: impl Into<String>) -> Self {
        Response {
            status: 302,
            headers: vec![("Location".into(), path.into())],
            body: vec![],
        }
    }
    pub fn not_found() -> Self {
        Response::with_type(404, "text/plain; charset=utf-8", b"not found".to_vec())
    }
    pub fn bad_request() -> Self {
        Response::with_type(400, "text/plain; charset=utf-8", b"bad request".to_vec())
    }
    pub fn too_many() -> Self {
        Response::with_type(429, "text/plain; charset=utf-8", b"slow down".to_vec())
    }
    pub fn with_type(status: u16, ctype: &str, body: Vec<u8>) -> Self {
        Response {
            status,
            headers: vec![("Content-Type".into(), ctype.into())],
            body,
        }
    }
}

pub fn serve(listener: TcpListener, handler: Handler) -> Result<()> {
    for conn in listener.incoming() {
        let mut sock = match conn {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(15)));
        let h = handler.clone();
        let peer = sock
            .peer_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_default();
        std::thread::spawn(move || {
            if let Ok(req) = read_request(&mut sock, &peer) {
                let resp = h(req);
                let _ = write_response(&mut sock, &resp);
            }
        });
    }
    Ok(())
}

fn read_request(sock: &mut TcpStream, peer: &str) -> Result<Request> {
    let mut reader = BufReader::new(sock.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.trim_end().split(' ');
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("");
    let (path, query) = match target.find('?') {
        Some(i) => (target[..i].to_string(), target[i + 1..].to_string()),
        None => (target.to_string(), String::new()),
    };
    let mut headers = Vec::new();
    loop {
        let mut hl = String::new();
        let n = reader.read_line(&mut hl)?;
        if n == 0 {
            break;
        }
        let hl = hl.trim_end();
        if hl.is_empty() {
            break;
        }
        if let Some(i) = hl.find(':') {
            headers.push((hl[..i].trim().to_string(), hl[i + 1..].trim().to_string()));
        }
    }
    Ok(Request {
        method,
        path,
        query,
        headers,
        peer: peer.to_string(),
    })
}

fn write_response(sock: &mut TcpStream, resp: &Response) -> Result<()> {
    let reason = match resp.status {
        200 => "OK",
        302 => "Found",
        400 => "Bad Request",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "OK",
    };
    write!(sock, "HTTP/1.1 {} {}\r\n", resp.status, reason)?;
    write!(sock, "Content-Length: {}\r\n", resp.body.len())?;
    for (k, v) in &resp.headers {
        write!(sock, "{k}: {v}\r\n")?;
    }
    write!(sock, "Connection: close\r\n\r\n")?;
    sock.write_all(&resp.body)?;
    Ok(())
}

/// Simple per-IP token bucket: 60 requests / 60 seconds. Locked by a
/// single Mutex (low QPS, contention is fine).
pub struct RateLimiter {
    inner: std::sync::Mutex<RateState>,
}
struct RateState {
    buckets: std::collections::HashMap<String, (std::time::Instant, u32)>,
    limit: u32,
}

impl RateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            inner: std::sync::Mutex::new(RateState {
                buckets: Default::default(),
                limit,
            }),
        }
    }
    pub fn allow(&self, ip: &str) -> bool {
        let mut s = self.inner.lock().unwrap();
        let now = std::time::Instant::now();
        let limit = s.limit;
        let entry = s.buckets.entry(ip.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0).as_secs() >= 60 {
            *entry = (now, 0);
        }
        if entry.1 >= limit {
            false
        } else {
            entry.1 += 1;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;

    #[test]
    fn rate_limiter() {
        let rl = RateLimiter::new(3);
        assert!(rl.allow("a"));
        assert!(rl.allow("a"));
        assert!(rl.allow("a"));
        assert!(!rl.allow("a"));
        assert!(rl.allow("b"));
    }

    #[test]
    fn roundtrip() {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let h = Arc::new(|_req: Request| Response::plain("hello"));
        std::thread::spawn({
            let l = l.try_clone().unwrap();
            move || serve(l, h).unwrap()
        });
        std::thread::sleep(Duration::from_millis(50));
        let mut s = TcpStream::connect(addr).unwrap();
        s.write_all(b"GET /healthz HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        s.read_to_string(&mut buf).unwrap();
        assert!(buf.starts_with("HTTP/1.1 200 OK"));
        assert!(buf.contains("hello"));
    }
}
