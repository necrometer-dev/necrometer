//! Native-only GitHub REST client. Auth: first non-empty of
//! `NECRO_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN`.

use super::pages::collect_pages;
use super::status::{classify_status, fetch_error, kind_from_user_doc, ProbeKind, StatusClass};
use super::{parse_repos, Repo};
use crate::error::Result;
use crate::http::Client;
use crate::metrics::SubjectKind;

const API: &str = "https://api.github.com";

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
        Ok(Self {
            client: Client::new()?,
            token,
        })
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    /// Kind + owned-repo list. Kind comes from `GET /users/{name}`
    /// (`type: Organization` vs User), not from which list endpoint 200s.
    pub fn resolve(&self, name: &str) -> std::result::Result<(SubjectKind, Vec<Repo>), FetchError> {
        let kind = self.probe_kind(name)?;
        let endpoint = match kind {
            ProbeKind::Org => format!("{API}/orgs/{name}/repos?type=all"),
            ProbeKind::SelfUser => {
                format!("{API}/user/repos?visibility=all&affiliation=owner")
            }
            ProbeKind::User => format!("{API}/users/{name}/repos"),
        };
        let repos = match self.fetch_all(&endpoint) {
            Err(FetchError::NotFound(_)) if kind == ProbeKind::Org => {
                self.fetch_all(&format!("{API}/users/{name}/repos"))?
            }
            r => r?,
        };
        Ok((kind.subject_kind(), repos))
    }

    pub fn resolve_repos(&self, name: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        self.resolve(name).map(|(_, r)| r)
    }

    fn probe_kind(&self, name: &str) -> std::result::Result<ProbeKind, FetchError> {
        let r = self
            .client
            .get(&format!("{API}/users/{name}"), self.token.as_deref())
            .map_err(|e| FetchError::Upstream(format!("{e}")))?;
        match classify_status(r.status) {
            StatusClass::Ok => {
                let me = self.self_login();
                Ok(kind_from_user_doc(&r.body, name, me.as_deref()))
            }
            StatusClass::NotFound => Err(FetchError::NotFound(name.to_string())),
            _ => Err(fetch_error(
                r.status,
                &format!("{API}/users/{name}"),
                &r.body,
            )),
        }
    }

    fn self_login(&self) -> Option<String> {
        if !self.has_token() {
            return None;
        }
        let r = self
            .client
            .get(&format!("{API}/user"), self.token.as_deref())
            .ok()?;
        if r.status != 200 {
            return None;
        }
        crate::json::parse(&r.body)
            .ok()?
            .get("login")?
            .as_str()
            .map(String::from)
    }

    fn fetch_all(&self, base: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        let sep = if base.contains('?') { '&' } else { '?' };
        collect_pages(|page| {
            let url = format!("{base}{sep}per_page=100&page={page}");
            self.get_page(&url)
        })
    }

    fn get_page(&self, url: &str) -> std::result::Result<Vec<Repo>, FetchError> {
        let r = self
            .client
            .get(url, self.token.as_deref())
            .map_err(|e| FetchError::Upstream(format!("{e}")))?;
        match classify_status(r.status) {
            StatusClass::Ok => {
                parse_repos(&r.body).map_err(|e| FetchError::Upstream(format!("{e}")))
            }
            _ => Err(fetch_error(r.status, url, &r.body)),
        }
    }
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
