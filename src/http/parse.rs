//! HTTP/1.1 response + URL parsing for the rustls client.

use super::Response;
use crate::error::{Error, Result};

pub(super) fn parse_response(raw: &[u8]) -> Result<Response> {
    let split = find_subslice(raw, b"\r\n\r\n")
        .ok_or_else(|| Error::Http("no header terminator".into()))?;
    let head = &raw[..split];
    let body = &raw[split + 4..];
    let head_str =
        std::str::from_utf8(head).map_err(|_| Error::Http("bad utf-8 in head".into()))?;
    let mut lines = head_str.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| Error::Http("no status line".into()))?;
    let mut parts = status_line.split(' ');
    let _http = parts.next();
    let status: u16 = parts
        .next()
        .ok_or_else(|| Error::Http("no status".into()))?
        .parse()
        .map_err(|_| Error::Http("bad status".into()))?;
    let mut next: Option<String> = None;
    let mut is_chunked = false;
    let mut content_length: Option<usize> = None;
    for line in lines {
        if let Some(rest) = line.strip_prefix("Link:") {
            let s = rest.trim();
            if let Some(url) = s.split(';').next() {
                let url = url.trim().trim_matches('<').trim_matches('>');
                if s.contains("rel=\"next\"") || s.contains("rel=next") {
                    next = Some(url.to_string());
                }
            }
        } else if let Some(rest) = line.strip_prefix("Transfer-Encoding:") {
            if rest.trim().eq_ignore_ascii_case("chunked") {
                is_chunked = true;
            }
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
        let line_end =
            find_subslice(&body[i..], b"\r\n").ok_or_else(|| Error::Http("bad chunk".into()))?;
        let size_str = std::str::from_utf8(&body[i..i + line_end])
            .map_err(|_| Error::Http("bad utf-8 in chunk".into()))?;
        let size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|_| Error::Http("bad chunk size".into()))?;
        i += line_end + 2;
        if size == 0 {
            break;
        }
        if i + size > body.len() {
            return Err(Error::Http("chunk overruns body".into()));
        }
        out.extend_from_slice(&body[i..i + size]);
        i += size;
        if i + 2 <= body.len() {
            i += 2;
        }
    }
    Ok(out)
}

pub(super) fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

pub(super) fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| Error::Http("only https supported".into()))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.find(':') {
        Some(i) => {
            let p: u16 = authority[i + 1..]
                .parse()
                .map_err(|_| Error::Http("bad port".into()))?;
            (&authority[..i], p)
        }
        None => (authority, 443),
    };
    Ok((host.to_string(), port, path.to_string()))
}

pub(super) fn response_complete(raw: &[u8]) -> bool {
    let Some(split) = find_subslice(raw, b"\r\n\r\n") else {
        return false;
    };
    let head = &raw[..split];
    let body = &raw[split + 4..];
    let Ok(head_str) = std::str::from_utf8(head) else {
        return false;
    };
    let mut chunked = false;
    let mut len: Option<usize> = None;
    for line in head_str.split("\r\n").skip(1) {
        if let Some(rest) = line.strip_prefix("Transfer-Encoding:") {
            chunked = rest.trim().eq_ignore_ascii_case("chunked");
        } else if let Some(rest) = line.strip_prefix("Content-Length:") {
            len = rest.trim().parse().ok();
        }
    }
    if chunked {
        return find_subslice(body, b"0\r\n\r\n").is_some()
            || find_subslice(body, b"0\n\n").is_some();
    }
    match len {
        Some(n) => body.len() >= n,
        None => false,
    }
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

    #[test]
    fn complete_on_content_length() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        assert!(response_complete(raw));
        let short = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n{}";
        assert!(!response_complete(short));
        let r = parse_response(raw).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, "{}");
    }
}
