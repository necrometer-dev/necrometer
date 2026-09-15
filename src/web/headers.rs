//! Response builders with security headers.

use crate::http_server::Response;

pub fn svg_response(svg: String) -> Response {
    let mut r = Response::svg(svg);
    r.headers.push(("Cache-Control".into(), "public, max-age=14400".into()));
    r.headers.push(("X-Content-Type-Options".into(), "nosniff".into()));
    r
}

pub fn html_response(html: String) -> Response {
    let mut r = Response::html(html);
    r.headers.push(("Cache-Control".into(), "public, max-age=300".into()));
    r.headers.push(("X-Content-Type-Options".into(), "nosniff".into()));
    r.headers.push(("Referrer-Policy".into(), "no-referrer".into()));
    r.headers.push(("X-Frame-Options".into(), "DENY".into()));
    r.headers.push((
        "Content-Security-Policy".into(),
        "default-src 'self'; script-src 'self' 'unsafe-inline'; \
         style-src 'self' 'unsafe-inline'; connect-src https://api.github.com; \
         img-src 'self' data:; font-src 'self'".into(),
    ));
    r
}