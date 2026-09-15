//! Error type for the whole crate. Hand-rolled — no anyhow/thiserror.
//!
//! Display is human-readable. No From blanket impls — callers convert
//! with `?` and `map_err(|e| Error::Io(e))` explicitly. This keeps the
//! dep graph at zero.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Http(String),
    Tls(String),
    Json(String),
    NotFound(String),
    Upstream(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Http(s) => write!(f, "http: {s}"),
            Error::Tls(s) => write!(f, "tls: {s}"),
            Error::Json(s) => write!(f, "json: {s}"),
            Error::NotFound(s) => write!(f, "not found: {s}"),
            Error::Upstream(s) => write!(f, "upstream: {s}"),
            Error::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
