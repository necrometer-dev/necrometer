//! Native-only GitHub REST client. Replaces reqwest's role in the
//! engine. Auth: first non-empty of `NECRO_TOKEN`, `GH_TOKEN`,
//! `GITHUB_TOKEN`. Resolution: orgs vs users vs the token's own
//! account (sees private).

use super::{parse_repos, Repo};
use crate::error::Result;
use crate::http::Client;
use crate::json::parse;

const API: &str = "https://api.github.com";
const MAX_PAGES: u32 = 10;

#[derive(Debug)]
pub enum FetchError {
    NotFound(String),
    Upstream(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::NotFound(s) => write!(f, "no such user or org: {s}"),
            FetchError::Upstream(s) => write!(f, "github: {s}"),
        }
    }
}

pub struct GitHub {
    client: Client,
    token: Option<String>,
}

impl GitHub {
    pub fn new() -> Result<Self> {
        let token = ["NECRO_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"]
            .iter()
            .find_map(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()))
            .map(|s| s.trim().to_string());
        let client = Client::new()?;
        Ok(Self { client, token })
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub fn resolve_repos(&self, name: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        let kind = self.resolve_kind(name);
        let endpoint = match kind {
            Kind::Org => format!("{API}/orgs/{}/repos?type=all", name),
            Kind::SelfUser => format!("{API}/user/repos?visibility=all&affiliation=owner"),
            Kind::User => format!("{API}/users/{}/repos", name),
        };
        match self.fetch_all(&endpoint) {
            Err(FetchError::NotFound(_)) if kind == Kind::Org => {
                self.fetch_all(&format!("{API}/users/{name}/repos"))
            }
            r => r,
        }
    }

    fn resolve_kind(&self, name: &str) -> Kind {
        if !self.has_token() {
            return Kind::User;
        }
        // Probe /users/{n} first; type == "Organization" → Org.
        let probe = format!("{API}/users/{name}");
        if let Ok(r) = self.client.get(&probe, self.token.as_deref()) {
            if r.status == 200 {
                if let Ok(v) = parse(&r.body) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("Organization") {
                        return Kind::Org;
                    }
                }
            }
        }
        // Otherwise: check if the token's own user matches.
        if let Ok(r) = self
            .client
            .get(&format!("{API}/user"), self.token.as_deref())
        {
            if r.status == 200 {
                if let Ok(v) = parse(&r.body) {
                    if let Some(login) = v.get("login").and_then(|x| x.as_str()) {
                        if login.eq_ignore_ascii_case(name) {
                            return Kind::SelfUser;
                        }
                    }
                }
            }
        }
        Kind::User
    }

    fn fetch_all(&self, base: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        // First page sequential; subsequent pages in two parallel
        // batches (2..=4, then 5..=MAX_PAGES) to bound latency.
        let sep = if base.contains('?') { '&' } else { '?' };
        let mut pages = vec![format!("{base}{sep}per_page=100&page=1")];
        for p in 2..=MAX_PAGES {
            pages.push(format!("{base}{sep}per_page=100&page={p}"));
        }

        let first = self.get_page(&pages[0])?;
        let total_pages = if first.len() < 100 {
            1
        } else {
            // Heuristic: try fetching all remaining pages in parallel,
            // stop at the first one that returns < 100 or 404.
            let mut all = vec![first];
            for batch in pages[1..].chunks(4) {
                let results: Vec<_> = std::thread::scope(|s| {
                    batch
                        .iter()
                        .map(|url| s.spawn(|| self.get_page(url)))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|h| {
                            h.join()
                                .unwrap_or(Err(FetchError::Upstream("worker".into())))
                        })
                        .collect()
                });
                let mut short = false;
                for r in results {
                    match r {
                        Ok(rs) => {
                            if rs.len() < 100 {
                                short = true;
                            }
                            all.push(rs);
                        }
                        Err(FetchError::NotFound(_)) => {
                            short = true;
                        }
                        Err(e) => return Err(e),
                    }
                }
                if short {
                    break;
                }
            }
            return Ok(all.into_iter().flatten().collect());
        };
        let _ = total_pages;
        Ok(first)
    }

    fn get_page(&self, url: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        match self.client.get(url, self.token.as_deref()) {
            Ok(r) if r.status == 404 => Err(FetchError::NotFound(url.to_string())),
            Ok(r) if (200..300).contains(&r.status) => {
                parse_repos(&r.body).map_err(|e| FetchError::Upstream(format!("{e}")))
            }
            Ok(r) => Err(FetchError::Upstream(format!(
                "GET {url} -> {}: {}",
                r.status,
                head(&r.body)
            ))),
            Err(e) => Err(FetchError::Upstream(format!("{e}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Org,
    SelfUser,
    User,
}

fn head(s: &str) -> String {
    s.chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::super::routes::SubjectKind;

    #[test]
    fn kind_prefix() {
        assert_eq!(SubjectKind::User.prefix(), "u");
        assert_eq!(SubjectKind::Org.prefix(), "org");
    }
}
