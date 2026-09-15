//! The right-side stats panel: subject name, index, title, flavor,
//! fate breakdown, stranded stars, oldest corpse, stillborn count,
//! and the EKG heartbeat (delegates to ekg::render).

use super::ekg;
use super::escape::esc_text;
use super::palette::{Palette, FONT_CREEPSTER, FONT_SERIF};
use crate::metrics::Reading;

pub fn render(reading: &Reading, p: &Palette) -> String {
    let mut s = String::new();
    let x = 200.0;

    s.push_str(&format!(
        "<text x=\"{x}\" y=\"33\" font-size=\"22\" letter-spacing=\"1.5\" fill=\"{c}\" class=\"font-creepster\" filter=\"url(#glow-violet)\" font-family=\"{font}\">@{subj}</text>",
        x = x, c = p.violet, subj = esc_text(&format!("@{}", reading.subject)), font = FONT_CREEPSTER
    ));

    if reading.total == 0 {
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"65\" font-size=\"14\" fill=\"{c}\" class=\"font-creepster\">no repos — nothing to bury</text>",
            x = x, c = p.dim
        ));
        return s;
    }

    let idx_glow = if reading.index >= 80 {
        " filter=\"url(#glow-blood)\""
    } else if reading.index >= 50 {
        " filter=\"url(#glow-candle)\""
    } else {
        " filter=\"url(#glow-phos)\""
    };
    s.push_str(&format!(
        "<text x=\"{x}\" y=\"68\" font-size=\"34\" font-weight=\"700\" fill=\"{c}\" class=\"font-mono\"{g}>{n}%</text>",
        x = x, c = p.accent, g = idx_glow, n = reading.index
    ));

    let necro_x = if reading.index >= 100 {
        x + 82.0
    } else if reading.index >= 10 {
        x + 70.0
    } else {
        x + 56.0
    };
    s.push_str(&format!(
        "<text x=\"{x}\" y=\"62\" font-size=\"12\" letter-spacing=\"1\" fill=\"{c}\" class=\"font-mono\">necrotic</text>",
        x = necro_x, c = p.dim
    ));

    s.push_str(&format!(
        "<text x=\"{x}\" y=\"88\" font-size=\"16\" letter-spacing=\"1\" fill=\"{c}\" class=\"font-creepster\" filter=\"url(#glow-candle)\" font-family=\"{font}\">{title}</text>",
        x = x, c = p.candle, font = FONT_CREEPSTER, title = esc_text(&reading.title)
    ));
    s.push_str(&format!(
        "<text x=\"{x}\" y=\"102\" font-size=\"11\" font-style=\"italic\" fill=\"{c}\" class=\"font-serif\" font-family=\"{font}\">{flavor}</text>",
        x = x, c = p.dim, font = FONT_SERIF, flavor = esc_text(&reading.flavor)
    ));

    let fate_line: Vec<String> = ["alive", "cooling", "cold", "dead", "buried"]
        .iter()
        .zip(reading.counts.iter())
        .filter(|(_, n)| **n > 0)
        .map(|(l, n)| format!("{n} {l}"))
        .collect();
    if !fate_line.is_empty() {
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"119\" font-size=\"11\" fill=\"{c}\">{fate}</text>",
            x = x,
            c = p.text,
            fate = esc_text(&fate_line.join(" · "))
        ));
    }

    let mut y = 133.0;
    if reading.stars_stranded > 0 {
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-size=\"10.5\" fill=\"{c}\">{n} stars stranded on dead repos</text>",
            x = x, y = y, c = p.candle, n = reading.stars_stranded
        ));
        y += 13.5;
    }
    if let Some((name, days)) = &reading.oldest_corpse {
        let name = if name.chars().count() > 20 {
            format!("{}…", name.chars().take(19).collect::<String>())
        } else {
            name.clone()
        };
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-size=\"10.5\" fill=\"{c}\">oldest corpse: {n} ({d}d)</text>",
            x = x, y = y, c = p.dim, n = esc_text(&name), d = days
        ));
        y += 13.5;
    }
    let mut bits = Vec::new();
    if reading.stillborn > 0 {
        bits.push(format!("{} stillborn", reading.stillborn));
    }
    if let Some(d) = reading.days_since_any_push {
        bits.push(if d == 0 {
            "signs of life today".to_string()
        } else {
            format!("last sign of life: {d}d ago")
        });
    }
    if !bits.is_empty() {
        let col = if reading.days_since_any_push == Some(0) {
            p.phos
        } else {
            p.dim
        };
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-size=\"10.5\" fill=\"{col}\">{bits}</text>",
            x = x,
            y = y,
            col = col,
            bits = esc_text(&bits.join(" · "))
        ));
    }

    s.push_str(&ekg::render(reading.index, p));
    s
}

#[cfg(test)]
mod tests {
    use super::super::palette::palette;
    use super::*;
    use crate::metrics::{Reading, SubjectKind};

    #[test]
    fn zero_repos_shows_message() {
        let r = Reading {
            subject: "x".into(),
            kind: SubjectKind::User,
            total: 0,
            counts: [0; 5],
            index: 0,
            title: "g".into(),
            flavor: "f".into(),
            stillborn: 0,
            stars_stranded: 0,
            oldest_corpse: None,
            days_since_any_push: None,
            low_sample: true,
            entries: vec![],
        };
        let p = palette(0);
        let svg = render(&r, &p);
        assert!(svg.contains("no repos"));
    }
}
