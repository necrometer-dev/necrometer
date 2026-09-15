//! Necro-metrics: decay tiers, the necrosis index, and tier titles.

pub mod analyze;
pub mod summary;
pub mod titles;

use crate::time::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Corpse {
    pub name: String,
    pub url: String,
    pub created_at: Utc,
    pub last_activity: Utc,
    pub days_idle: u32,
    pub stars: u64,
    pub fate: Fate,
    pub stillborn: bool,
}

#[derive(Debug, Clone)]
pub struct Reading {
    pub subject: String,
    pub kind: SubjectKind,
    /// Necrosis index, 0..=100. 0 = everything alive, 100 = full graveyard.
    pub index: u8,
    pub title: String,
    pub flavor: String,
    pub counts: [u32; 5],
    pub total: u32,
    pub stars_stranded: u64,
    pub oldest_corpse: Option<(String, u32)>,
    pub stillborn: u32,
    pub days_since_any_push: Option<u32>,
    pub low_sample: bool,
    pub entries: Vec<Corpse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn analyze(subject: &str, kind: SubjectKind, repos: &[crate::github::Repo]) -> Reading {
    analyze::analyze(subject, kind, repos)
}
