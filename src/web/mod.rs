//! Web service: routes, TTL cache, rate limiting, card endpoints.
//!
//! State flows through an Arc<AppState> captured by the handler. The
//! router returns a closure that captures the state directly — no
//! thread-local needed.

pub mod css;
pub mod headers;
pub mod pages;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::card;
use crate::github::client::{FetchError, GitHub};
use crate::github::routes::SubjectKind;
use crate::http_server::{serve as serve_http, Handler, RateLimiter, Request, Response};
use crate::metrics::{analyze, Reading};

const CACHE_TTL: Duration = Duration::from_secs(4 * 3600);
const LIMIT: u32 = 60;

pub struct AppState {
    gh: GitHub,
    cache: Mutex<HashMap<String, (Instant, Reading)>>,
    limiter: RateLimiter,
}

pub fn build() -> crate::error::Result<Arc<AppState>> {
    Ok(Arc::new(AppState {
        gh: GitHub::new()?,
        cache: Mutex::new(HashMap::new()),
        limiter: RateLimiter::new(LIMIT),
    }))
}

/// Build the request handler. Closes over the AppState.
pub fn handler(st: Arc<AppState>) -> Handler {
    Arc::new(move |req: Request| route(&st, req))
}

fn route(st: &AppState, req: Request) -> Response {
    let path = req.path.as_str();
    let limit_paths = ["/u/", "/org/", "/go"];
    if limit_paths.iter().any(|p| path.starts_with(p)) && !st.limiter.allow(&req.peer) {
        return Response::too_many();
    }
    if path == "/" || path.is_empty() {
        return Response::html(pages::landing());
    }
    if path == "/healthz" {
        return Response::plain("ok");
    }
    if path == "/go" || path.starts_with("/go?") {
        return handle_go(&req);
    }
    if let Some(rest) = path.strip_prefix("/u/") {
        return handle_subject(st, rest, SubjectKind::User);
    }
    if let Some(rest) = path.strip_prefix("/org/") {
        return handle_subject(st, rest, SubjectKind::Org);
    }
    Response::not_found()
}

fn handle_go(req: &Request) -> Response {
    let subject = req
        .query
        .split('&')
        .find_map(|kv| kv.strip_prefix("subject="))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !crate::github::routes::is_valid_subject(&subject) {
        return Response::bad_request();
    }
    Response::redirect(format!("/u/{subject}"))
}

fn handle_subject(st: &AppState, rest: &str, kind: SubjectKind) -> Response {
    let (name, want_svg) = match rest.strip_suffix(".svg") {
        Some(n) => (n.to_string(), true),
        None => (rest.to_string(), false),
    };
    if !crate::github::routes::is_valid_subject(&name) {
        return Response::bad_request();
    }
    let reading = match read_or_fetch(st, &name, kind) {
        Ok(r) => r,
        Err(FetchError::NotFound(n)) => {
            return Response::html(pages::not_found(&format!(
                "no such {}: {n}",
                kind_str(kind)
            )));
        }
        Err(FetchError::Upstream(e)) => {
            eprintln!("github fetch failed: {e}");
            return Response::with_type(
                502,
                "text/html; charset=utf-8",
                pages::upstream_error().into_bytes(),
            );
        }
    };
    if want_svg {
        headers::svg_response(card::render(&reading))
    } else {
        headers::html_response(pages::subject_page(&reading))
    }
}

fn read_or_fetch(
    st: &AppState,
    name: &str,
    kind: SubjectKind,
) -> std::result::Result<Reading, FetchError> {
    let key = format!("{}:{name}", kind.prefix());
    {
        let cache = st.cache.lock().unwrap();
        if let Some((ts, r)) = cache.get(&key) {
            if ts.elapsed() < CACHE_TTL {
                return Ok(r.clone());
            }
        }
    }
    let repos = st.gh.resolve_repos(name)?;
    let reading = analyze(name, kind, &repos);
    st.cache
        .lock()
        .unwrap()
        .insert(key, (Instant::now(), reading.clone()));
    Ok(reading)
}

fn kind_str(kind: SubjectKind) -> &'static str {
    match kind {
        SubjectKind::User => "user",
        SubjectKind::Org => "org",
    }
}

/// Entry point: bind, build state, run forever.
pub fn run(bind: &str) -> crate::error::Result<()> {
    let listener = std::net::TcpListener::bind(bind)?;
    let st = build()?;
    let h = handler(st);
    serve_http(listener, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_disambiguates() {
        let a = format!("{}:foo", SubjectKind::User.prefix());
        let b = format!("{}:foo", SubjectKind::Org.prefix());
        assert_ne!(a, b);
    }
}
