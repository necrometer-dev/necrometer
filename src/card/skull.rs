//! The little skull buddy mascot in the top-right. The face mirrors
//! the reading: happy/ok/sad/dead.

use super::geometry::fmt;
use super::palette::Palette;

pub fn render(index: u8, p: &Palette) -> String {
    let (sx, sy) = (450.0_f64, 42.0_f64);
    let face = match index {
        0..=14 => "happy",
        15..=59 => "ok",
        60..=79 => "sad",
        _ => "dead",
    };
    let mut s = String::new();
    if face == "happy" {
        s.push_str(&format!(
            "<ellipse cx=\"{sx}\" cy=\"{y}\" rx=\"6\" ry=\"2\" fill=\"none\" stroke=\"{c}\" stroke-width=\"1.5\" filter=\"url(#glow-candle)\"/>",
            sx = sx, y = fmt(sy - 17.0), c = p.candle
        ));
    }
    s.push_str(&format!(
        "<path d=\"M {x1},{y1} A 10.5 10.5 0 1 1 {x2},{y1} L {x2},{y2} Q {x2},{y3} {x3},{y3} L {x4},{y3} Q {x1},{y3} {x1},{y2} Z\" fill=\"{c}\" stroke=\"#2d2642\" stroke-width=\"0.8\"/>",
        x1 = fmt(sx - 10.5), y1 = fmt(sy + 3.0),
        x2 = fmt(sx + 10.5),
        y2 = fmt(sy + 6.0),
        y3 = fmt(sy + 10.5),
        x3 = fmt(sx + 6.5), x4 = fmt(sx - 6.5),
        c = p.text
    ));
    s.push_str(&format!(
        "<path d=\"M {a},{y} v -3 M {b},{y} v -3 M {cc},{y} v -3\" stroke=\"{c}\" stroke-width=\"1\"/>",
        a = fmt(sx - 3.5), b = fmt(sx), cc = fmt(sx + 3.5), y = fmt(sy + 10.5), c = p.inset
    ));
    for ex in [sx - 4.0, sx + 4.0] {
        let ey = sy - 0.5;
        let eye = match face {
            "happy" => format!(
                "<path d=\"M {a},{y1} Q {m},{y2} {b},{y1}\" stroke=\"{c}\" stroke-width=\"1.5\" fill=\"none\" stroke-linecap=\"round\"/>",
                a = fmt(ex - 2.4), y1 = fmt(ey + 1.0), m = fmt(ex), y2 = fmt(ey - 2.2), b = fmt(ex + 2.4), c = p.inset
            ),
            "dead" => format!(
                "<path d=\"M {a},{y} l 4,4 M {b},{y} l -4,4\" stroke=\"{c}\" stroke-width=\"1.3\" stroke-linecap=\"round\"/>",
                a = fmt(ex - 2.0), b = fmt(ex + 2.0), y = fmt(ey - 2.0), c = p.inset
            ),
            _ => format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"2.3\" fill=\"{c}\"/>",
                x = fmt(ex), y = fmt(ey), c = p.inset
            ),
        };
        s.push_str(&eye);
    }
    s.push_str(&format!(
        "<path d=\"M {x},{y} l -1.4,-2.2 l 2.8,0 Z\" fill=\"{c}\"/>",
        x = fmt(sx),
        y = fmt(sy + 4.5),
        c = p.inset
    ));
    if face == "sad" {
        s.push_str(&format!(
            "<path d=\"M {a},{y2} Q {m},{y1} {b},{y2}\" stroke=\"{c}\" stroke-width=\"1\" fill=\"none\"/>",
            a = fmt(sx - 2.5), y2 = fmt(sy + 8.4), m = fmt(sx), y1 = fmt(sy + 6.8), b = fmt(sx + 2.5), c = p.inset
        ));
    }
    if face == "happy" {
        s.push_str(&format!(
            "<ellipse cx=\"{a}\" cy=\"{y}\" rx=\"1.8\" ry=\"1.1\" fill=\"{c}\" opacity=\"0.8\"/><ellipse cx=\"{b}\" cy=\"{y}\" rx=\"1.8\" ry=\"1.1\" fill=\"{c}\" opacity=\"0.8\"/>",
            a = fmt(sx - 6.8), b = fmt(sx + 6.8), y = fmt(sy + 4.0), c = p.blood
        ));
    }
    if face == "dead" {
        s.push_str(&format!(
            "<path d=\"M {x},{y} l 3,3.5 l -1.8,2.5\" stroke=\"{c}\" stroke-width=\"0.9\" fill=\"none\"/>",
            x = fmt(sx - 4.0), y = fmt(sy - 8.5), c = p.dim
        ));
    }
    format!("<g>{s}</g>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_face_branch() {
        let p = super::super::palette::palette(0);
        assert!(render(0, &p).contains("<ellipse")); // halo for happy
        assert!(!render(50, &p).contains("halo")); // no halo for ok
        let p100 = super::super::palette::palette(100);
        assert!(render(100, &p100).contains("l 3,3.5")); // crack for dead
    }
}
