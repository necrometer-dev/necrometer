//! The little graveyard under the gauge. Stones for the dead (half
//! size for stillborn), flowers if all alive.

use super::geometry::{fmt, pt};
use super::palette::Palette;
use crate::metrics::{Fate, Reading};

pub fn render(reading: &Reading, p: &Palette) -> String {
    let gy = 180.0_f64;
    let mut s = format!(
        "<line x1=\"24\" y1=\"{gy}\" x2=\"166\" y2=\"{gy}\" stroke=\"{c}\" stroke-width=\"1\"/>",
        gy = fmt(gy),
        c = p.border
    );
    let mut dead: Vec<&crate::metrics::Corpse> = reading
        .entries
        .iter()
        .filter(|e| e.fate != Fate::Alive)
        .collect();
    dead.sort_by(|a, b| {
        b.stillborn
            .cmp(&a.stillborn)
            .then(b.days_idle.cmp(&a.days_idle))
    });
    if dead.is_empty() {
        for i in 0..3 {
            let fx = 42.0 + i as f64 * 30.0;
            s.push_str(&format!(
                "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{c}\" stroke-width=\"1.2\"/>",
                x1 = fmt(fx), y1 = fmt(gy), x2 = fmt(fx), y2 = fmt(gy - 7.0), c = p.phos
            ));
            for k in 0..5 {
                let (px, py) = pt(fx, gy - 9.5, 2.4, k as f64 * 72.0 + 90.0);
                let color = if i % 2 == 1 { p.blood } else { p.candle };
                s.push_str(&format!(
                    "<circle cx=\"{x}\" cy=\"{y}\" r=\"1.7\" fill=\"{c}\"/>",
                    x = fmt(px),
                    y = fmt(py),
                    c = color
                ));
            }
            s.push_str(&format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"1.4\" fill=\"{c}\"/>",
                x = fmt(fx),
                y = fmt(gy - 9.5),
                c = p.text
            ));
        }
        return format!("<g>{s}</g>");
    }
    for (i, e) in dead.iter().take(6).enumerate() {
        let (w, h) = if e.stillborn {
            (8.0, 7.0)
        } else {
            (12.0, 11.0)
        };
        let x = 28.0 + i as f64 * 22.0;
        let top = gy - h;
        s.push_str(&format!(
            "<path d=\"M {x1},{y1} L {x2},{y2} A {wr} {wr} 0 0 1 {x3},{y2} L {x4},{y1} Z\" fill=\"#28213b\" stroke=\"#3d3356\" stroke-width=\"0.8\"/>",
            x1 = fmt(x), y1 = fmt(gy),
            x2 = fmt(x), y2 = fmt(top + w / 2.0),
            wr = w / 2.0,
            x3 = fmt(x + w),
            x4 = fmt(x + w)
        ));
        s.push_str(&format!(
            "<path d=\"M {x1},{y1} h 3.6 M {x2},{y2} v 3.6\" stroke=\"{c}\" stroke-width=\"0.9\"/>",
            x1 = fmt(x + w / 2.0 - 1.8),
            y1 = fmt(top + w / 2.0 + 1.2),
            x2 = fmt(x + w / 2.0),
            y2 = fmt(top + w / 2.0 - 0.6),
            c = p.inset
        ));
    }
    if dead.len() > 6 {
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-size=\"9\" fill=\"{c}\" class=\"font-mono\">+{n}</text>",
            x = fmt(28.0 + 6.0 * 22.0 - 6.0), y = fmt(gy - 2.0), c = p.dim, n = dead.len() - 6
        ));
    }
    format!("<g>{s}</g>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{Corpse, Fate, Reading, SubjectKind};
    use crate::time::Utc;

    fn sample(index: u8) -> Reading {
        let mut r = Reading {
            subject: "x".into(),
            kind: SubjectKind::User,
            total: 0,
            counts: [0; 5],
            index,
            title: "t".into(),
            flavor: "f".into(),
            stillborn: 0,
            stars_stranded: 0,
            oldest_corpse: None,
            days_since_any_push: None,
            low_sample: false,
            entries: vec![],
        };
        if index >= 50 {
            let now = Utc::now();
            r.entries.push(Corpse {
                name: "a".into(),
                url: "".into(),
                created_at: Utc(now.0 - 86400 * 1000),
                last_activity: Utc(now.0 - 86400 * 900),
                days_idle: 900,
                stars: 0,
                fate: Fate::Dead,
                stillborn: false,
            });
        }
        r
    }

    #[test]
    fn flowers_when_all_alive() {
        let p = super::super::palette::palette(0);
        let svg = render(&sample(0), &p);
        assert!(svg.contains("circle"));
    }
}
