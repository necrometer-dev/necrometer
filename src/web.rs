//! Web service: routes, TTL cache, card endpoints, and HTML pages.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use tokio::sync::RwLock;
use tracing::warn;

use crate::card;
use crate::github::{FetchError, GitHub, Repo};
use crate::metrics::{analyze, Reading, SubjectKind};

const CACHE_TTL: Duration = Duration::from_secs(4 * 3600);
const CARD_MAX_AGE: &str = "public, max-age=14400";

pub struct AppState {
    gh: GitHub,
    cache: RwLock<HashMap<String, (Instant, Arc<Reading>)>>,
}

pub fn router() -> Result<Router> {
    let state = Arc::new(AppState {
        gh: GitHub::new()?,
        cache: RwLock::new(HashMap::new()),
    });
    Ok(Router::new()
        .route("/", get(landing))
        .route("/go", get(go))
        .route("/healthz", get(|| async { "ok" }))
        .route("/u/{*rest}", get(user_handler))
        .route("/org/{*rest}", get(org_handler))
        .with_state(state))
}

async fn user_handler(
    State(st): State<Arc<AppState>>,
    Path(rest): Path<String>,
) -> Response {
    subject_handler(st, rest, SubjectKind::User).await
}

async fn org_handler(
    State(st): State<Arc<AppState>>,
    Path(rest): Path<String>,
) -> Response {
    subject_handler(st, rest, SubjectKind::Org).await
}

async fn subject_handler(st: Arc<AppState>, rest: String, kind: SubjectKind) -> Response {
    let (name, want_svg) = match rest.strip_suffix(".svg") {
        Some(n) => (n.to_string(), true),
        None => (rest, false),
    };
    if name.is_empty() || name.contains('/') || name.starts_with('.') {
        return Html(not_found_page("bad subject").into_string()).into_response();
    }

    match reading_for(&st, kind, &name).await {
        Ok(reading) if want_svg => svg_response(&card::render(&reading)),
        Ok(reading) => page_response(subject_page(&reading)),
        Err(FetchError::NotFound(n)) => Html(
            not_found_page(&format!("no such {}: {n}", kind_label(kind))).into_string(),
        )
        .into_response(),
        Err(FetchError::Upstream(e)) => {
            warn!(error = %e, "github fetch failed");
            (
                StatusCode::BAD_GATEWAY,
                Html(page_markup("upstream error", html! {
                    p { "GitHub said no: " (e.to_string()) }
                    p { a href="/" { "back to the morgue" } }
                }).into_string()),
            )
                .into_response()
        }
    }
}

async fn reading_for(
    st: &Arc<AppState>,
    kind: SubjectKind,
    name: &str,
) -> Result<Arc<Reading>, FetchError> {
    let key = format!("{}:{name}", kind.prefix());
    if let Some((ts, r)) = st.cache.read().await.get(&key) {
        if ts.elapsed() < CACHE_TTL {
            return Ok(r.clone());
        }
    }

    let repos: Vec<Repo> = match kind {
        SubjectKind::User => st.gh.user_repos(name).await?,
        SubjectKind::Org => st.gh.org_repos(name).await?,
    };
    let reading = Arc::new(analyze(name, kind, &repos));
    st.cache
        .write()
        .await
        .insert(key, (Instant::now(), reading.clone()));
    Ok(reading)
}

fn kind_label(kind: SubjectKind) -> &'static str {
    match kind {
        SubjectKind::User => "user",
        SubjectKind::Org => "org",
    }
}

fn svg_response(svg: &str) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml; charset=utf-8"),
            (header::CACHE_CONTROL, CARD_MAX_AGE),
            (header::HeaderName::from_static("x-content-type-options"), "nosniff"),
        ],
        svg.to_string(),
    )
        .into_response()
}

fn page_response(m: Markup) -> Response {
    (
        [(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=300"))],
        Html(m.into_string()),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
struct GoQuery {
    subject: Option<String>,
}

async fn go(Query(q): Query<GoQuery>) -> Redirect {
    let subject = q.subject.unwrap_or_default().trim().to_string();
    Redirect::temporary(&format!("/u/{subject}"))
}

async fn landing() -> Html<String> {
    Html(landing_markup().into_string())
}

fn landing_markup() -> Markup {
    page_markup("necrometer — how dead are your repos?", html! {
        h1 { "THE NECROMETER" }
        p class="tag" { "Measures how dead your GitHub repos are. Embeds in your README, haunts your profile." }
        form action="/go" method="get" {
            input name="subject" placeholder="github username or org" autofocus;
            button type="submit" { "check the pulse" }
        }
        p class="hint" { "embed: " code { "[![Necrometer](https://necrometer.dev/u/YOU.svg)](https://necrometer.dev/u/YOU)" } }
        h2 { "specimens" }
        img src="/u/torvalds.svg" alt="torvalds necrometer";
        img src="/org/rust-lang.svg" alt="rust-lang necrometer";
    })
}

fn subject_page(r: &Reading) -> Markup {
    let base = "https://necrometer.dev";
    let path = format!("/{}/{}", r.kind.prefix(), r.subject);
    let snippet = format!("[![Necrometer]({base}{path}.svg)]({base}{path})");
    page_markup(&format!("{} — necrometer reading", r.subject), html! {
        p { a href="/" { "← the morgue" } }
        h1 { "@" (r.subject) }
        img src=(format!("{path}.svg")) alt="necrometer card";
        h2 { (format!("{}% necrotic — {}", r.index, r.title)) }
        @if r.total > 0 {
            p class="dim" {
                (r.total) " repos examined · "
                (r.stars_stranded) " stars stranded"
                @if r.stillborn > 0 { " · " (r.stillborn) " stillborn" }
            }
        }
        @if !r.corpses.is_empty() {
            h2 { "the graveyard" }
            table {
                tr { th { "repo" } th { "idle" } th { "stars" } th { "fate" } }
                @for c in &r.corpses {
                    tr class=(format!("fate-{}", c.fate.label())) {
                        td { a href=(c.url) { (c.name) }
                            @if c.stillborn { span class="stillborn" { " stillborn" } } }
                        td { (c.days_idle) "d" }
                        td { (c.stars) }
                        td { (c.fate.label()) }
                    }
                }
            }
        }
        h2 { "haunt your README" }
        pre { (snippet) }
        form action="/go" method="get" {
            input name="subject" placeholder="check another";
            button type="submit" { "exhume" }
        }
    })
}

fn not_found_page(msg: &str) -> Markup {
    page_markup("not found", html! {
        h1 { "empty plot" }
        p { (msg) }
        form action="/go" method="get" {
            input name="subject" placeholder="try someone else";
            button type="submit" { "dig" }
        }
    })
}

fn page_markup(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                meta property="og:title" content=(title);
                style { (PreEscaped(CSS)) }
            }
            body { main { (body) } }
        }
    }
}

const CSS: &str = r#"
body { background:#0d1117; color:#e6edf3; font-family:-apple-system,'Segoe UI',Roboto,Helvetica,Arial,sans-serif; margin:0 }
main { max-width:640px; margin:48px auto; padding:0 16px }
h1 { letter-spacing:3px; font-size:26px }
h2 { font-size:16px; color:#8b949e; letter-spacing:1px; text-transform:uppercase; margin-top:32px }
a { color:#a371f7 }
.tag { color:#8b949e }
.dim { color:#8b949e }
img { display:block; margin:12px 0; max-width:100% }
form { margin:24px 0 }
input { background:#161b22; border:1px solid #30363d; color:#e6edf3; padding:10px 12px; border-radius:6px; width:240px; font-size:14px }
button { background:#a371f7; color:#0d1117; border:0; padding:10px 16px; border-radius:6px; font-weight:600; cursor:pointer }
pre { background:#161b22; border:1px solid #30363d; padding:12px; border-radius:6px; overflow-x:auto; font-size:12px }
code { background:#161b22; padding:2px 5px; border-radius:4px; font-size:12px }
.hint { color:#8b949e; font-size:13px }
table { width:100%; border-collapse:collapse; font-size:14px }
th { text-align:left; color:#8b949e; font-weight:500; padding:6px 8px; border-bottom:1px solid #30363d }
td { padding:6px 8px; border-bottom:1px solid #21262d }
.fate-dead td:last-child, .fate-buried td:last-child { color:#f85149 }
.fate-cold td:last-child { color:#d29922 }
.fate-cooling td:last-child { color:#58a6ff }
.stillborn { color:#8b949e; font-size:11px; font-style:italic }
"#;
