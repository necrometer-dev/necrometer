//! Necro-metrics: decay tiers, the necrosis index, and tier titles.

use chrono::{DateTime, Utc};

use crate::github::Repo;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Fate {
    Alive,
    Cooling,
    Cold,
    Dead,
    Buried,
}

impl Fate {
    pub fn weight(self) -> f64 {
        match self {
            Fate::Alive => 0.0,
            Fate::Cooling => 0.33,
            Fate::Cold => 0.66,
            Fate::Dead => 1.0,
            Fate::Buried => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Fate::Alive => "alive",
            Fate::Cooling => "cooling",
            Fate::Cold => "cold",
            Fate::Dead => "dead",
            Fate::Buried => "buried",
        }
    }
}

/// One repo's vital record — kept for all repos, alive or dead (the
/// graveyard strip and lifelines need the living ones too).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Corpse {
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub days_idle: u32,
    pub stars: u64,
    pub fate: Fate,
    pub stillborn: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub subject: String,
    pub kind: SubjectKind,
    /// Necrosis index, 0..=100. 0 = everything alive, 100 = full graveyard.
    pub index: u8,
    pub title: String,
    pub flavor: String,
    /// Counts by fate, in enum order.
    pub counts: [u32; 5],
    pub total: u32,
    pub stars_stranded: u64,
    pub oldest_corpse: Option<(String, u32)>,
    pub stillborn: u32,
    pub days_since_any_push: Option<u32>,
    pub low_sample: bool,
    /// All owned non-fork repos, every fate.
    pub entries: Vec<Corpse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubjectKind {
    User,
    Org,
}

impl SubjectKind {
    pub fn prefix(self) -> &'static str {
        match self {
            SubjectKind::User => "u",
            SubjectKind::Org => "org",
        }
    }
}

pub fn analyze(subject: &str, kind: SubjectKind, repos: &[Repo]) -> Reading {
    let now = Utc::now();
    let owned: Vec<&Repo> = repos.iter().filter(|r| !r.fork).collect();
    let total = owned.len() as u32;

    let mut counts = [0u32; 5];
    let mut entries = Vec::new();
    let mut weight_sum = 0.0;
    let mut stars_stranded = 0u64;
    let mut stillborn = 0u32;
    let mut last_push: Option<DateTime<Utc>> = None;

    for repo in &owned {
        let fate = fate_of(repo, now);
        counts[fate as usize] += 1;
        weight_sum += fate.weight();

        let last_activity = repo.pushed_at.unwrap_or(repo.created_at);
        if last_push.is_none_or(|p| last_activity > p) {
            last_push = Some(last_activity);
        }

        let days_idle = (now - last_activity).num_days().max(0) as u32;
        let born_dead = repo.pushed_at.is_none()
            || (last_activity - repo.created_at).num_hours() <= 24;
        if fate != Fate::Alive {
            stars_stranded += repo.stargazers_count;
            if born_dead {
                stillborn += 1;
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
        title: title_for(index, total),
        flavor: flavor_for(index, total),
        counts,
        total,
        stars_stranded,
        oldest_corpse,
        stillborn,
        days_since_any_push: last_push.map(|p| (now - p).num_days().max(0) as u32),
        low_sample,
        entries,
    }
}

fn fate_of(repo: &Repo, now: DateTime<Utc>) -> Fate {
    if repo.archived {
        return Fate::Buried;
    }
    let last_activity = repo.pushed_at.unwrap_or(repo.created_at);
    match (now - last_activity).num_days() {
        d if d < 30 => Fate::Alive,
        d if d < 180 => Fate::Cooling,
        d if d < 730 => Fate::Cold,
        _ => Fate::Dead,
    }
}

fn title_for(index: u8, total: u32) -> String {
    if total == 0 {
        return "Ghost — nothing to bury".into();
    }
    if total < 3 {
        return "Insufficient Corpses".into();
    }
    match index {
        0..=14 => "The Maintainer",
        15..=34 => "Healthy Churn",
        35..=59 => "Serial Starter",
        60..=79 => "Graveyard Keeper",
        _ => "Repo Necromancer",
    }
    .into()
}

fn flavor_for(index: u8, total: u32) -> String {
    if total == 0 {
        return "no repos. no pulse. nothing.".into();
    }
    if total < 3 {
        return "not enough bodies to judge".into();
    }
    match index {
        0..=14 => "nothing dies here. suspicious.",
        15..=34 => "a few corpses, like everyone",
        35..=59 => "starts things. finishes? unclear.",
        60..=79 => "more graves than gardens",
        _ => "not a profile — a cemetery",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(pushed_days_ago: i64, archived: bool, fork: bool, stars: u64) -> Repo {
        let now = Utc::now();
        Repo {
            name: "x".into(),
            pushed_at: Some(now - chrono::Duration::days(pushed_days_ago)),
            created_at: now - chrono::Duration::days(pushed_days_ago + 400),
            archived,
            fork,
            stargazers_count: stars,
            size: 10,
            description: None,
            html_url: String::new(),
        }
    }

    #[test]
    fn all_alive_is_zero() {
        let r = analyze("t", SubjectKind::User, &[repo(1, false, false, 0), repo(5, false, false, 0), repo(9, false, false, 0)]);
        assert_eq!(r.index, 0);
        assert_eq!(r.title, "The Maintainer");
    }

    #[test]
    fn forks_excluded() {
        let r = analyze("t", SubjectKind::User, &[repo(1, false, false, 0), repo(2000, false, true, 0)]);
        assert_eq!(r.total, 1);
        assert_eq!(r.index, 0);
    }

    #[test]
    fn archived_is_buried() {
        let r = analyze("t", SubjectKind::User, &[repo(900, true, false, 0)]);
        assert_eq!(r.counts[Fate::Buried as usize], 1);
        assert_eq!(r.index, 100);
    }
}
