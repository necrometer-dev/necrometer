//! SVG card renderer. Orchestrator only — every visual concept lives
//! in its own file. Public surface is `pub fn render(&Reading) -> String`.

pub mod chassis;
pub mod easter;
pub mod ekg;
pub mod escape;
pub mod gauge;
pub mod geometry;
pub mod graveyard;
pub mod palette;
pub mod skull;
pub mod stats;

use crate::metrics::Reading;

pub fn render(reading: &Reading) -> String {
    let p = palette::palette(reading.index);
    let mut s = String::with_capacity(48000);
    s.push_str(&chassis::svg_open());
    s.push_str(&chassis::defs());
    s.push_str(&chassis::chassis(&p));
    s.push_str(&graveyard::render(reading, &p));
    s.push_str(&gauge::render(reading.index, &p));
    s.push_str(&stats::render(reading, &p));
    s.push_str(&skull::render(reading.index, &p));
    s.push_str(&easter::render(reading.index));
    s.push_str(&chassis::watermark(&p));
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{Reading, SubjectKind};

    fn sample(index: u8, total: u32) -> Reading {
        Reading {
            subject: "test-user".into(),
            kind: SubjectKind::User,
            total,
            counts: [total, 0, 0, 0, 0],
            index,
            title: "The Test Subject".into(),
            flavor: "testing edge cases".into(),
            stillborn: 0,
            stars_stranded: 0,
            oldest_corpse: None,
            days_since_any_push: Some(0),
            low_sample: false,
            entries: Vec::new(),
        }
    }

    #[test]
    fn contains_creepster_and_meta() {
        let r = sample(10, 5);
        let svg = render(&r);
        assert!(svg.contains("@font-face"));
        assert!(svg.contains("Creepster"));
        assert!(svg.contains("necrometer.dev"));
        assert!(svg.contains("@test-user"));
    }

    #[test]
    fn zero_repos_message() {
        let mut r = sample(0, 0);
        r.counts = [0; 5];
        let svg = render(&r);
        assert!(svg.contains("no repos"));
    }

    #[test]
    fn easter_eggs_appear() {
        let svg0 = render(&sample(0, 5));
        assert!(svg0.contains("glow-candle"));
        let svg90 = render(&sample(90, 5));
        assert!(svg90.contains("polyline"));
    }

    #[test]
    fn escapes_user_data() {
        let mut r = sample(10, 1);
        r.subject = "foo&<bar>\"baz".into();
        r.title = "A & B".into();
        r.flavor = "1 < 2 > 0".into();
        let svg = render(&r);
        assert!(svg.contains("foo&amp;&lt;bar&gt;&quot;baz"));
        assert!(!svg.contains("<bar>"));
    }
}