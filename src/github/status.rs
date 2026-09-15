//! HTTP status classification and user/org kind from a GitHub user JSON
//! document. Pure — no network — so tests don't need a token.

use super::client::FetchError;
use crate::json::parse;
use crate::metrics::SubjectKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    Org,
    SelfUser,
    User,
}

impl ProbeKind {
    pub fn subject_kind(self) -> SubjectKind {
        match self {
            ProbeKind::Org => SubjectKind::Org,
            ProbeKind::SelfUser | ProbeKind::User => SubjectKind::User,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Ok,
    NotFound,
    RateLimited,
    Other(u16),
}

pub fn classify_status(status: u16) -> StatusClass {
    match status {
        200..=299 => StatusClass::Ok,
        404 => StatusClass::NotFound,
        403 | 429 => StatusClass::RateLimited,
        s => StatusClass::Other(s),
    }
}

pub fn fetch_error(status: u16, url: &str, body: &str) -> FetchError {
    match classify_status(status) {
        StatusClass::NotFound => FetchError::NotFound(url.to_string()),
        StatusClass::RateLimited => FetchError::Upstream("GitHub rate limit hit".into()),
        StatusClass::Ok => FetchError::Upstream(format!("unexpected ok on {url}")),
        StatusClass::Other(s) => FetchError::Upstream(format!(
            "GET {url} -> {s}: {}",
            body.chars().take(200).collect::<String>()
        )),
    }
}

/// `body` is the JSON from `GET /users/{name}`. `self_login` is the
/// token's own login, if known.
pub fn kind_from_user_doc(body: &str, name: &str, self_login: Option<&str>) -> ProbeKind {
    let Ok(v) = parse(body) else {
        return ProbeKind::User;
    };
    if v.get("type").and_then(|x| x.as_str()) == Some("Organization") {
        return ProbeKind::Org;
    }
    if let Some(me) = self_login {
        if me.eq_ignore_ascii_case(name) {
            return ProbeKind::SelfUser;
        }
    }
    ProbeKind::User
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_is_not_missing_user() {
        for s in [403_u16, 429] {
            let e = fetch_error(s, "https://api.github.com/users/x/repos", "");
            let msg = format!("{e}");
            assert!(!msg.contains("no such user"), "{msg}");
            assert!(msg.to_lowercase().contains("rate limit"), "{msg}");
        }
    }

    #[test]
    fn four_oh_four_is_missing() {
        let e = fetch_error(404, "https://api.github.com/users/nope", "");
        assert!(format!("{e}").contains("no such user"));
    }

    #[test]
    fn org_from_type_field() {
        let k = kind_from_user_doc(
            r#"{"login":"rust-lang","type":"Organization"}"#,
            "rust-lang",
            None,
        );
        assert_eq!(k, ProbeKind::Org);
        assert_eq!(k.subject_kind(), SubjectKind::Org);
    }

    #[test]
    fn user_from_type_field() {
        let k = kind_from_user_doc(r#"{"login":"torvalds","type":"User"}"#, "torvalds", None);
        assert_eq!(k, ProbeKind::User);
        assert_eq!(k.subject_kind(), SubjectKind::User);
    }

    #[test]
    fn self_user_when_login_matches_token() {
        let k = kind_from_user_doc(r#"{"login":"me","type":"User"}"#, "me", Some("me"));
        assert_eq!(k, ProbeKind::SelfUser);
        assert_eq!(k.subject_kind(), SubjectKind::User);
    }
}
