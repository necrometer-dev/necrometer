//! Stillborn detection: born dead = no push, OR pushed within 24h of
//! creation.

use crate::github::Repo;

pub fn stillborn(repo: &Repo) -> bool {
    let last = repo.pushed_at.unwrap_or(repo.created_at);
    repo.pushed_at.is_none() || ((last.0 - repo.created_at.0) <= 24 * 3600)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Utc;

    fn repo_with(pushed: Option<Utc>, created: Utc) -> Repo {
        Repo {
            name: "x".into(),
            pushed_at: pushed,
            created_at: created,
            archived: false,
            fork: false,
            stargazers_count: 0,
            html_url: String::new(),
        }
    }

    #[test]
    fn no_push_is_stillborn() {
        let now = Utc::now();
        let r = repo_with(None, Utc(now.0 - 86400));
        assert!(stillborn(&r));
    }

    #[test]
    fn pushed_within_24h_is_stillborn() {
        let now = Utc::now();
        let r = repo_with(Some(Utc(now.0 - 3600)), Utc(now.0 - 86400));
        assert!(stillborn(&r));
    }

    #[test]
    fn pushed_after_24h_is_alive() {
        let now = Utc::now();
        let r = repo_with(Some(Utc(now.0 - 86400 * 30)), Utc(now.0 - 86400 * 400));
        assert!(!stillborn(&r));
    }
}
