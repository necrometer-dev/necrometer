//! The arc gauge: 5 zones, the dial face, the swinging needle.

use super::geometry::{fmt, pts};
use super::palette::{Palette, FONT_CREEPSTER};

pub fn render(index: u8, p: &Palette) -> String {
    let (cx, cy, r) = (95.0_f64, 132.0_f64, 62.0_f64);
    let mut g = String::new();

    g.push_str("<path d=\"M 33.0,132.0 A 62 62 0 0 1 157.0,129.8\" stroke=\"#191428\" stroke-width=\"11\" fill=\"none\" stroke-linecap=\"round\"/>");

    for (i, color) in p.zones.iter().enumerate() {
        let a0 = 180.0 - i as f64 * 36.0;
        let a1 = a0 - 34.0;
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
        x = cx - r - 2.0, y = cy + 14.0, c = p.phos
    ));
    g.push_str(&format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" font-size=\"12\" fill=\"{c}\" class=\"font-mono\" filter=\"url(#glow-blood)\">☠</text>",
        x = cx + r + 2.0, y = cy + 14.0, c = p.blood
    ));

    let theta = 180.0 - index as f64 * 1.8;
    let swing = (180.0 - theta).max(30.0);
    let needle_glow = if index < 60 {
        " filter=\"url(#glow-blood)\""
    } else {
        ""
    };
    g.push_str(&format!(
        "<g><animateTransform attributeName=\"transform\" type=\"rotate\" values=\"-{s} {cx} {cy};4 {cx} {cy};0 {cx} {cy}\" keyTimes=\"0;0.8;1\" dur=\"1.1s\" fill=\"freeze\"/>",
        s = fmt(swing), cx = cx, cy = cy
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
    use super::super::palette::palette;
    use super::*;

    #[test]
    fn emits_five_zone_arcs() {
        let p = palette(50);
        let svg = render(50, &p);
        assert!(svg.matches("<path").count() >= 6);
        assert!(svg.contains("<polygon"));
    }
}
