//! The arc gauge: 5 zones, the dial face, the swinging needle.
//!
//! `pt()` is clock-style: 0° = up, 90° = right, 180° = down, 270° = left.
//! The dial is the *top* semicircle, 0% at the left (270°) through up
//! (0°) to 100% at the right (90°).

use super::geometry::{fmt, pts};
use super::palette::{Palette, FONT_CREEPSTER};

const CX: f64 = 95.0;
const CY: f64 = 132.0;
const R: f64 = 62.0;
/// Left end of the top semicircle (0%).
const A_LEFT: f64 = 270.0;
/// Right end of the top semicircle (100%).
const A_RIGHT: f64 = 90.0;
const ZONE_GAP: f64 = 2.0;
const N_ZONES: f64 = 5.0;

fn zone_angles(i: usize) -> (f64, f64) {
    let sweep = 180.0;
    let span = (sweep - ZONE_GAP * (N_ZONES - 1.0)) / N_ZONES;
    let a0 = A_LEFT + i as f64 * (span + ZONE_GAP);
    (a0, a0 + span)
}

fn needle_angle(index: u8) -> f64 {
    A_LEFT + index as f64 * 1.8
}

pub fn render(index: u8, p: &Palette) -> String {
    let (cx, cy, r) = (CX, CY, R);
    let mut g = String::new();

    // Dark rail: two quarter-arcs so a 180° sweep isn't ambiguous.
    let left = pts(cx, cy, r, A_LEFT);
    let top = pts(cx, cy, r, 0.0);
    let right = pts(cx, cy, r, A_RIGHT);
    g.push_str(&format!(
        "<path d=\"M {left} A {r} {r} 0 0 1 {top} A {r} {r} 0 0 1 {right}\" stroke=\"#191428\" stroke-width=\"11\" fill=\"none\" stroke-linecap=\"round\"/>"
    ));

    for (i, color) in p.zones.iter().enumerate() {
        let (a0, a1) = zone_angles(i);
        let glow = match i {
            0 => " filter=\"url(#glow-phos)\"",
            4 => " filter=\"url(#glow-blood)\"",
            _ => "",
        };
        g.push_str(&format!(
            "<path d=\"M {p0} A {r} {r} 0 0 1 {p1}\" stroke=\"{c}\" stroke-width=\"9\" fill=\"none\" stroke-linecap=\"round\"{glow}/>",
            p0 = pts(cx, cy, r, a0), p1 = pts(cx, cy, r, a1), r = r, c = color, glow = glow
        ));
    }

    g.push_str(&format!(
        "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{c}\" stroke-width=\"1\"/>",
        x1 = cx - r - 8.0,
        y1 = cy,
        x2 = cx + r + 8.0,
        y2 = cy,
        c = p.border
    ));

    g.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" font-size=\"11\" fill=\"{c}\" class=\"font-mono\" filter=\"url(#glow-phos)\">0</text>",
        x = cx - r, y = cy + 14.0, c = p.phos
    ));
    g.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" font-size=\"12\" fill=\"{c}\" class=\"font-mono\" filter=\"url(#glow-blood)\">☠</text>",
        x = cx + r, y = cy + 14.0, c = p.blood
    ));

    let theta = needle_angle(index);
    let travel = index as f64 * 1.8;
    let needle_glow = if index < 60 {
        " filter=\"url(#glow-blood)\""
    } else {
        ""
    };
    g.push_str(&format!(
        "<g><animateTransform attributeName=\"transform\" type=\"rotate\" values=\"-{t} {cx} {cy};4 {cx} {cy};0 {cx} {cy}\" keyTimes=\"0;0.8;1\" dur=\"1.1s\" fill=\"freeze\"/>",
        t = fmt(travel), cx = cx, cy = cy
    ));
    g.push_str(&format!(
        "<polygon points=\"{tip} {bl} {br}\" fill=\"{c}\"{glow}/>",
        tip = pts(cx, cy, r - 12.0, theta),
        bl = pts(cx, cy, 3.8, theta + 90.0),
        br = pts(cx, cy, 3.8, theta - 90.0),
        c = p.needle,
        glow = needle_glow
    ));
    g.push_str(&format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"5.5\" fill=\"{bg}\" stroke=\"{c}\" stroke-width=\"1.8\"/></g>",
        cx = cx, cy = cy, bg = p.inset, c = p.needle
    ));

    g.push_str(&format!(
        "<text x=\"{cx}\" y=\"{y}\" text-anchor=\"middle\" font-size=\"14\" letter-spacing=\"2\" fill=\"{c}\" class=\"font-creepster\" filter=\"url(#glow-violet)\" font-family=\"{font}\">NECROMETER</text>",
        cx = cx, y = cy + 32.0, c = p.dim, font = FONT_CREEPSTER
    ));
    g
}

#[cfg(test)]
mod tests {
    use super::super::geometry::pt;
    use super::super::palette::palette;
    use super::*;

    #[test]
    fn emits_five_zone_arcs() {
        let p = palette(50);
        let svg = render(50, &p);
        assert!(svg.matches("<path").count() >= 6);
        assert!(svg.contains("<polygon"));
    }

    #[test]
    fn rail_meets_the_diameter_ends() {
        let svg = render(0, &palette(0));
        assert!(svg.contains("33.0,132.0"), "left end: {svg}");
        assert!(svg.contains("157.0,132.0"), "right end: {svg}");
        assert!(!svg.contains("129.8"), "old off-center right end leaked");
    }

    #[test]
    fn zones_sit_on_the_top_semicircle() {
        for i in 0..5 {
            let (a0, a1) = zone_angles(i);
            let (_, y0) = pt(CX, CY, R, a0);
            let (_, y1) = pt(CX, CY, R, a1);
            assert!(y0 <= CY + 0.6, "zone {i} start y {y0} below dial");
            assert!(y1 <= CY + 0.6, "zone {i} end y {y1} below dial");
        }
        let (a0, _) = zone_angles(0);
        let (_, a1) = zone_angles(4);
        assert!((a0 - A_LEFT).abs() < 0.01);
        // last zone ends at 90° (450° unwrapped)
        assert!((a1 - (A_LEFT + 180.0)).abs() < 0.01);
    }

    #[test]
    fn dump_gauges_for_visual_check() {
        let dir = match std::env::var("DUMP_GAUGE") {
            Ok(d) => d,
            Err(_) => return,
        };
        for i in [0_u8, 25, 50, 75, 100] {
            let svg = crate::card::render(&crate::metrics::Reading {
                subject: format!("i{i}"),
                kind: crate::metrics::SubjectKind::User,
                total: 10,
                counts: [2, 2, 2, 2, 2],
                index: i,
                title: "Gauge Check".into(),
                flavor: "alignment".into(),
                stillborn: 0,
                stars_stranded: 0,
                oldest_corpse: None,
                days_since_any_push: Some(0),
                low_sample: false,
                entries: Vec::new(),
            });
            std::fs::write(format!("{dir}/gauge-{i}.svg"), svg).unwrap();
        }
    }

    #[test]
    fn needle_runs_left_to_right() {
        let (x0, y0) = pt(CX, CY, R - 12.0, needle_angle(0));
        let (x50, y50) = pt(CX, CY, R - 12.0, needle_angle(50));
        let (x100, y100) = pt(CX, CY, R - 12.0, needle_angle(100));
        assert!(x0 < CX - 20.0, "0% tip not left {x0}");
        assert!((y0 - CY).abs() < 1.0, "0% tip not on diameter {y0}");
        assert!((x50 - CX).abs() < 1.0, "50% tip not up-center {x50}");
        assert!(y50 < CY - 20.0, "50% tip not up {y50}");
        assert!(x100 > CX + 20.0, "100% tip not right {x100}");
        assert!((y100 - CY).abs() < 1.0, "100% tip not on diameter {y100}");
    }
}
