//! Minimal GitHub REST client — repo listing for users, orgs, and single repos.
//! `Repo` itself is pure data and compiles for wasm; the client is native-only
//! (the browser path fetches via JS and hands us the JSON).

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::header::{self, HeaderMap, HeaderValue};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{Client, StatusCode};

#[cfg(not(target_arch = "wasm32"))]
const API: &str = "https://api.github.com";
#[cfg(not(target_arch = "wasm32"))]
const MAX_PAGES: u32 = 10;

#[derive(Debug, Clone, Deserialize)]
pub struct Repo {
    pub name: String,
    /// Null on repos that never received a commit — born dead.
    pub pushed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub stargazers_count: u64,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub html_url: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum FetchError {
    NotFound(String),
    Upstream(anyhow::Error),
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::NotFound(s) => write!(f, "no such user or org: {s}"),
            FetchError::Upstream(e) => write!(f, "github: {e}"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for FetchError {}

#[cfg(not(target_arch = "wasm32"))]
pub struct GitHub {
    client: Client,
    authed: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl GitHub {
    pub fn new() -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("necrometer/0.1 (https://necrometer.dev)"),
        );
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        // First defined token wins — the site and generated workflows agree on these.
        let mut authed = false;
        for var in ["NECRO_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"] {
            if let Ok(tok) = std::env::var(var) {
                let val = format!("Bearer {}", tok.trim());
                headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&val)?);
                authed = true;
                break;
            }
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self { client, authed })
    }

    pub fn has_token(&self) -> bool {
        self.authed
    }

    /// Pick the right listing endpoint for a subject. `/users/{n}/repos` is
    /// public-only even with a token, so: orgs get `/orgs/{n}/repos?type=all`,
    /// the token's own account gets `/user/repos` (sees private), everyone
    /// else gets the public listing.
    pub async fn resolve_repos(&self, name: &str) -> Result<Vec<Repo>, FetchError> {
        let base = self.resolve_endpoint(name).await;
        match self.get_all(&base, name).await {
            // An org that 404s on /orgs/{n} (hidden to this token) still answers
            // on the public listing — report what we can see instead of lying.
            Err(FetchError::NotFound(_)) if base.contains("/orgs/") => {
                self.get_all(&format!("{API}/users/{name}/repos"), name).await
            }
            r => r,
        }
    }

    async fn resolve_endpoint(&self, name: &str) -> String {
        if self.has_token() {
            // Detect orgs via /users/{n}.type — /orgs/{n} 404s for orgs hidden
            // from this token even when the account itself is public.
            if let Ok(resp) = self.client.get(format!("{API}/users/{name}")).send().await {
                if let Ok(j) = resp.json::<serde_json::Value>().await {
                    if j["type"].as_str() == Some("Organization") {
                        return format!("{API}/orgs/{name}/repos?type=all");
                    }
                }
            }
            if let Ok(resp) = self.client.get(format!("{API}/user")).send().await {
                if let Ok(j) = resp.json::<serde_json::Value>().await {
                    if j["login"]
                        .as_str()
                        .is_some_and(|l| l.eq_ignore_ascii_case(name))
                    {
                        return format!("{API}/user/repos?visibility=all&affiliation=owner");
                    }
                }
            }
        }
        format!("{API}/users/{name}/repos")
    }

    /// First page sequential; once we know there are more, fetch the rest in
    /// two parallel batches (2-4, then 5-10). 10 sequential round-trips on a
    /// 1000-repo account was the worst case.
    async fn get_all(&self, base: &str, subject: &str) -> Result<Vec<Repo>, FetchError> {
        let sep = if base.contains('?') { '&' } else { '?' };
        let page_url = |page: u32| format!("{base}{sep}per_page=100&page={page}");
        let first = self.get_page(&page_url(1), subject).await?;
        let last = first.len() < 100;
        let mut out = first;
        if last {
            return Ok(out);
        }
        for batch in [2..=4u32, 5..=MAX_PAGES] {
            let mut set = tokio::task::JoinSet::new();
            for page in batch.clone() {
                let (client, url, subj) = (
                    self.client.clone(),
                    page_url(page),
                    subject.to_string(),
                );
                set.spawn(async move { fetch_page(&client, &url, &subj).await });
            }
            let mut short = false;
            while let Some(res) = set.join_next().await {
                let (page, repos) = res.map_err(|e| FetchError::Upstream(e.into()))??;
                if repos.len() < 100 {
                    short = true;
                }
                // `out` isn't sorted by page; order doesn't matter downstream.
                let _ = page;
                out.extend(repos);
            }
            if short {
                break;
            }
        }
        Ok(out)
    }

    async fn get_page(&self, url: &str, subject: &str) -> Result<Vec<Repo>, FetchError> {
        fetch_page(&self.client, url, subject).await.map(|(_, r)| r)
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_page(client: &Client, url: &str, subject: &str) -> Result<(u32, Vec<Repo>), FetchError> {
    let page: u32 = url
        .rsplit("page=")
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| FetchError::Upstream(e.into()))?;
    match resp.status() {
        StatusCode::NOT_FOUND => Err(FetchError::NotFound(subject.into())),
        s if !s.is_success() => {
            let body = resp.text().await.unwrap_or_default();
            Err(FetchError::Upstream(anyhow::anyhow!("GET {url} -> {s}: {body}")))
        }
        _ => {
            let repos: Vec<Repo> = resp
                .json()
                .await
                .map_err(|e| FetchError::Upstream(e.into()))?;
            Ok((page, repos))
        }
    }
}
