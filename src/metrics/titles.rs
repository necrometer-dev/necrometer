//! Tier titles and flavor lines. The only creative writing in the
//! engine — kept in one place so future PRs that touch the voice
//! land here.

pub fn title_for(index: u8, total: u32) -> String {
    if total == 0 { return "Ghost — nothing to bury".into(); }
    if total < 3 { return "Insufficient Corpses".into(); }
    match index {
        0..=14 => "The Maintainer",
        15..=34 => "Healthy Churn",
        35..=59 => "Serial Starter",
        60..=79 => "Graveyard Keeper",
        _ => "Repo Necromancer",
    }.into()
}

pub fn flavor_for(index: u8, total: u32) -> String {
    if total == 0 { return "no repos. no pulse. nothing.".into(); }
    if total < 3 { return "not enough bodies to judge".into(); }
    match index {
        0..=14 => "nothing dies here. suspicious.",
        15..=34 => "a few corpses, like everyone",
        35..=59 => "starts things. finishes? unclear.",
        60..=79 => "more graves than gardens",
        _ => "not a profile — a cemetery",
    }.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_when_empty() {
        assert_eq!(title_for(0, 0), "Ghost — nothing to bury");
        assert!(flavor_for(0, 0).contains("nothing"));
    }

    #[test]
    fn low_sample() {
        assert_eq!(title_for(50, 2), "Insufficient Corpses");
    }

    #[test]
    fn bands() {
        assert_eq!(title_for(0, 10), "The Maintainer");
        assert_eq!(title_for(34, 10), "Healthy Churn");
        assert_eq!(title_for(59, 10), "Serial Starter");
        assert_eq!(title_for(79, 10), "Graveyard Keeper");
        assert_eq!(title_for(100, 10), "Repo Necromancer");
    }
}