//! The `analyze()` function: walks repos, assigns fates, computes
//! index, finds oldest corpse.

use super::summary::stillborn;
use super::titles::{flavor_for, title_for};
use super::{Corpse, Fate, Reading, SubjectKind};
use crate::github::Repo;
use crate::time::Utc;

pub fn analyze(subject: &str, kind: SubjectKind, repos: &[Repo]) -> Reading {
    let now = Utc::now();
    let owned: Vec<&Repo> = repos.iter().filter(|r| !r.fork).collect();
    let total = owned.len() as u32;

    let mut counts = [0u32; 5];
    let mut entries = Vec::with_capacity(owned.len());
    let mut weight_sum = 0.0;
    let mut stars_stranded = 0u64;
    let mut stillborn_count = 0u32;
    let mut last_push: Option<Utc> = None;

    for repo in &owned {
        let fate = fate_of(repo, now);
        counts[fate as usize] += 1;
        weight_sum += fate.weight();

        let last_activity = repo.pushed_at.unwrap_or(repo.created_at);
        if last_push.is_none_or(|p| last_activity > p) {
            last_push = Some(last_activity);
        }

        let days_idle = (now - last_activity).num_days().max(0) as u32;
        let born_dead = stillborn(repo);
        if fate != Fate::Alive {
            stars_stranded += repo.stargazers_count;
            if born_dead {
                stillborn_count += 1;
            }
        }
        entries.push(Corpse {
            name: repo.name.clone(),
            url: repo.html_url.clone(),
            created_at: repo.created_at,
            last_activity,
            days_idle,
            stars: repo.stargazers_count,
            fate,
            stillborn: born_dead,
        });
    }

    let oldest_corpse = entries
        .iter()
        .filter(|c| c.fate != Fate::Alive)
        .max_by_key(|c| c.days_idle)
        .map(|c| (c.name.clone(), c.days_idle));

    let low_sample = total < 3;
    let index = if total == 0 {
        0
    } else {
        ((weight_sum / total as f64) * 100.0).round() as u8
    };

    Reading {
        subject: subject.to_string(),
        kind,
        index,
        title: title_for(index, total, kind),
        flavor: flavor_for(index, total, kind),
        counts,
        total,
        stars_stranded,
        oldest_corpse,
        stillborn: stillborn_count,
        days_since_any_push: last_push.map(|p| (now - p).num_days().max(0) as u32),
        low_sample,
        entries,
    }
}

pub fn fate_of(repo: &Repo, now: Utc) -> Fate {
    if repo.archived {
        return Fate::Buried;
    }
    let last_activity = repo.pushed_at.unwrap_or(repo.created_at);
    let d = (now - last_activity).num_days();
    match d {
        d if d < 30 => Fate::Alive,
        d if d < 180 => Fate::Cooling,
        d if d < 730 => Fate::Cold,
        _ => Fate::Dead,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(pushed_days_ago: i64, archived: bool, fork: bool, stars: u64) -> Repo {
        let now = Utc::now();
        Repo {
            name: "x".into(),
            pushed_at: Some(Utc(now.0 - pushed_days_ago * 86400)),
            created_at: Utc(now.0 - (pushed_days_ago + 400) * 86400),
            archived,
            fork,
            stargazers_count: stars,
            html_url: String::new(),
        }
    }

    #[test]
    fn all_alive_is_zero() {
        let r = analyze(
            "t",
            SubjectKind::User,
            &[
                repo(1, false, false, 0),
                repo(5, false, false, 0),
                repo(9, false, false, 0),
            ],
        );
        assert_eq!(r.index, 0);
        assert_eq!(r.title, "The Maintainer");
        let org = analyze(
            "o",
            SubjectKind::Org,
            &[
                repo(1, false, false, 0),
                repo(5, false, false, 0),
                repo(9, false, false, 0),
            ],
        );
        assert_eq!(org.title, "Healthy Churn");
    }

    #[test]
    fn forks_excluded() {
        let r = analyze(
            "t",
            SubjectKind::User,
            &[repo(1, false, false, 0), repo(2000, false, true, 0)],
        );
        assert_eq!(r.total, 1);
        assert_eq!(r.index, 0);
    }

    #[test]
    fn archived_is_buried() {
        let r = analyze("t", SubjectKind::User, &[repo(900, true, false, 0)]);
        assert_eq!(r.counts[Fate::Buried as usize], 1);
        assert_eq!(r.index, 100);
    }

    #[test]
    fn boundary_ages() {
        let now = Utc::now();
        let mut r = repo(29, false, false, 0);
        assert_eq!(fate_of(&r, now), Fate::Alive);
        r.pushed_at = Some(Utc(now.0 - 30 * 86400));
        assert_eq!(fate_of(&r, now), Fate::Cooling);
        r.pushed_at = Some(Utc(now.0 - 180 * 86400));
        assert_eq!(fate_of(&r, now), Fate::Cold);
        r.pushed_at = Some(Utc(now.0 - 730 * 86400));
        assert_eq!(fate_of(&r, now), Fate::Dead);
    }
}
