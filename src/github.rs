//! Minimal GitHub REST client — repo listing for users, orgs, and single repos.

use std::fmt;

use chrono::{DateTime, Utc};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

const API: &str = "https://api.github.com";
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

#[derive(Debug)]
pub enum FetchError {
    NotFound(String),
    Upstream(anyhow::Error),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::NotFound(s) => write!(f, "no such user or org: {s}"),
            FetchError::Upstream(e) => write!(f, "github: {e}"),
        }
    }
}

impl std::error::Error for FetchError {}

pub struct GitHub {
    client: Client,
}

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
        if let Ok(tok) = std::env::var("GITHUB_TOKEN") {
            let val = format!("Bearer {}", tok.trim());
            headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&val)?);
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self { client })
    }

    pub async fn user_repos(&self, user: &str) -> Result<Vec<Repo>, FetchError> {
        self.get_all(&format!("{API}/users/{user}/repos"), user).await
    }

    pub async fn org_repos(&self, org: &str) -> Result<Vec<Repo>, FetchError> {
        self.get_all(&format!("{API}/orgs/{org}/repos"), org).await
    }

    async fn get_all(&self, base: &str, subject: &str) -> Result<Vec<Repo>, FetchError> {
        let mut out = Vec::new();
        for page in 1..=MAX_PAGES {
            let url = format!("{base}?per_page=100&type=all&page={page}");
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| FetchError::Upstream(e.into()))?;
            match resp.status() {
                StatusCode::NOT_FOUND => return Err(FetchError::NotFound(subject.into())),
                s if !s.is_success() => {
                    let body = resp.text().await.unwrap_or_default();
                    return Err(FetchError::Upstream(anyhow::anyhow!(
                        "GET {url} -> {s}: {body}"
                    )));
                }
                _ => {}
            }
            let page_repos: Vec<Repo> = resp
                .json()
                .await
                .map_err(|e| FetchError::Upstream(e.into()))?;
            let last = page_repos.len() < 100;
            out.extend(page_repos);
            if last {
                break;
            }
        }
        Ok(out)
    }
}
