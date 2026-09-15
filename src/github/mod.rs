//! GitHub REST surface: `Repo` data, the native HTTP client, and path
//! validation. `Repo` is wasm-safe; the client is cfg-gated to native.

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
pub mod routes;

use crate::error::Result;
use crate::json::{parse, Value};
use crate::time::Utc;

#[derive(Debug, Clone)]
pub struct Repo {
    pub name: String,
    /// `None` means "born dead" — repo never received a commit.
    pub pushed_at: Option<Utc>,
    pub created_at: Utc,
    pub archived: bool,
    pub fork: bool,
    pub stargazers_count: u64,
    pub html_url: String,
}

pub fn parse_repos(json_text: &str) -> Result<Vec<Repo>> {
    let v = parse(json_text)?;
    let arr = match v {
        Value::Array(a) => a,
        _ => return Err(crate::error::Error::Json("expected array".into())),
    };
    let mut out = Vec::with_capacity(arr.len());
    for r in arr {
        out.push(parse_repo(&r)?);
    }
    Ok(out)
}

fn parse_repo(v: &Value) -> Result<Repo> {
    let name = v.get("name")
        .and_then(|x| x.as_str())
        .ok_or_else(|| crate::error::Error::Json("missing name".into()))?
        .to_string();
    let created_at = v.get("created_at")
        .and_then(|x| x.as_str())
        .ok_or_else(|| crate::error::Error::Json("missing created_at".into()))?;
    let created_at = Utc::parse_rfc3339(created_at)
        .ok_or_else(|| crate::error::Error::Json("bad created_at".into()))?;
    let pushed_at = v.get("pushed_at")
        .and_then(|x| x.as_str())
        .and_then(Utc::parse_rfc3339);
    let archived = v.get("archived").and_then(|x| x.as_bool()).unwrap_or(false);
    let fork = v.get("fork").and_then(|x| x.as_bool()).unwrap_or(false);
    let stargazers_count = v.get("stargazers_count")
        .and_then(|x| x.as_u64()).unwrap_or(0);
    let html_url = v.get("html_url")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Repo { name, pushed_at, created_at, archived, fork, stargazers_count, html_url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_repo() {
        let raw = r#"[{
            "name":"hello",
            "pushed_at":"2024-06-01T00:00:00Z",
            "created_at":"2023-01-01T00:00:00Z",
            "archived":false,"fork":false,"stargazers_count":7,
            "html_url":"https://github.com/x/hello"
        }]"#;
        let rs = parse_repos(raw).unwrap();
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].name, "hello");
        assert_eq!(rs[0].stargazers_count, 7);
        assert!(rs[0].pushed_at.is_some());
    }

    #[test]
    fn born_dead_when_no_pushed_at() {
        let raw = r#"[{"name":"x","pushed_at":null,"created_at":"2023-01-01T00:00:00Z","archived":false,"fork":false,"stargazers_count":0,"html_url":""}]"#;
        let rs = parse_repos(raw).unwrap();
        assert!(rs[0].pushed_at.is_none());
    }
}