//! Page walk for GitHub list endpoints. 30 × 100 = 3000 repos, matching
//! the site. Tested with a fake `get_page` — no network.

use super::client::FetchError;
use super::Repo;

pub const MAX_PAGES: u32 = 30;

pub fn collect_pages<F>(get_page: F) -> std::result::Result<Vec<Repo>, FetchError>
where
    F: Fn(u32) -> std::result::Result<Vec<Repo>, FetchError> + Sync,
{
    let first = get_page(1)?;
    if first.len() < 100 {
        return Ok(first);
    }
    let mut all = vec![first];
    let mut page = 2u32;
    while page <= MAX_PAGES {
        let end = (page + 3).min(MAX_PAGES);
        let batch: Vec<u32> = (page..=end).collect();
        let results: Vec<_> = std::thread::scope(|s| {
            batch
                .iter()
                .map(|p| s.spawn(|| get_page(*p)))
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
                Err(FetchError::NotFound(_)) => short = true,
                Err(e) => return Err(e),
            }
        }
        if short {
            break;
        }
        page = end + 1;
    }
    Ok(all.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Utc;

    fn n_repos(n: usize, prefix: &str) -> Vec<Repo> {
        let t = Utc::parse_rfc3339("2024-01-01T00:00:00Z").unwrap();
        (0..n)
            .map(|i| Repo {
                name: format!("{prefix}{i}"),
                pushed_at: Some(t),
                created_at: t,
                archived: false,
                fork: false,
                stargazers_count: 0,
                html_url: String::new(),
            })
            .collect()
    }

    #[test]
    fn walks_past_one_thousand() {
        // 12 full pages + a short 13th → 1250 bodies, past the old 10-page cap.
        let get = |p: u32| -> std::result::Result<Vec<Repo>, FetchError> {
            if p <= 12 {
                Ok(n_repos(100, &format!("p{p}-")))
            } else if p == 13 {
                Ok(n_repos(50, "tail-"))
            } else {
                panic!("fetched page {p} after a short page");
            }
        };
        let repos = collect_pages(get).unwrap();
        assert_eq!(repos.len(), 1250);
        assert!(repos.len() > 1000);
    }

    #[test]
    fn stops_on_short_first_page() {
        let get = |p: u32| {
            assert_eq!(p, 1);
            Ok(n_repos(3, "x"))
        };
        assert_eq!(collect_pages(get).unwrap().len(), 3);
    }

    #[test]
    fn rate_limit_fails_the_walk() {
        let get = |p: u32| {
            if p == 1 {
                Ok(n_repos(100, "a"))
            } else {
                Err(FetchError::Upstream("GitHub rate limit hit".into()))
            }
        };
        let e = collect_pages(get).unwrap_err();
        assert!(!format!("{e}").contains("no such user"));
    }
}
