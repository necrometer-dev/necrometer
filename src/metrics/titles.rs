//! Tier titles and flavor lines. The only creative writing in the
//! engine — kept in one place so future PRs that touch the voice
//! land here.

use super::SubjectKind;

pub fn title_for(index: u8, total: u32, kind: SubjectKind) -> String {
    if total == 0 {
        return "Ghost — nothing to bury".into();
    }
    if total < 3 {
        return "Insufficient Corpses".into();
    }
    match (index, kind) {
        (0..=14, SubjectKind::Org) => "Healthy Churn",
        (0..=14, SubjectKind::User) => "The Maintainer",
        (15..=34, _) => "Healthy Churn",
        (35..=59, _) => "Serial Starter",
        (60..=79, _) => "Graveyard Keeper",
        _ => "Repo Necromancer",
    }
    .into()
}

pub fn flavor_for(index: u8, total: u32, kind: SubjectKind) -> String {
    if total == 0 {
        return "no repos. no pulse. nothing.".into();
    }
    if total < 3 {
        return "not enough bodies to judge".into();
    }
    match (index, kind) {
        (0..=14, SubjectKind::Org) => "the org still has a pulse",
        (0..=14, SubjectKind::User) => "nothing dies here. suspicious.",
        (15..=34, _) => "a few corpses, like everyone",
        (35..=59, _) => "starts things. finishes? unclear.",
        (60..=79, _) => "more graves than gardens",
        _ => "not a profile — a cemetery",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::SubjectKind;

    #[test]
    fn ghost_when_empty() {
        assert_eq!(
            title_for(0, 0, SubjectKind::User),
            "Ghost — nothing to bury"
        );
        assert!(flavor_for(0, 0, SubjectKind::User).contains("nothing"));
    }

    #[test]
    fn low_sample() {
        assert_eq!(title_for(50, 2, SubjectKind::User), "Insufficient Corpses");
    }

    #[test]
    fn bands() {
        assert_eq!(title_for(0, 10, SubjectKind::User), "The Maintainer");
        assert_eq!(title_for(0, 10, SubjectKind::Org), "Healthy Churn");
        assert_eq!(title_for(34, 10, SubjectKind::User), "Healthy Churn");
        assert_eq!(title_for(59, 10, SubjectKind::User), "Serial Starter");
        assert_eq!(title_for(79, 10, SubjectKind::User), "Graveyard Keeper");
        assert_eq!(title_for(100, 10, SubjectKind::User), "Repo Necromancer");
    }
}
