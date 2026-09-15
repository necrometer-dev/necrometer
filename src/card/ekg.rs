//! The EKG heartbeat line at the bottom of the stats panel.
//! Flatlines for a dead reading, pulses with a heart for alive.

use super::geometry::fmt;
use super::palette::Palette;

pub fn render(index: u8, p: &Palette) -> String {
    let (x0, w, base) = (200.0_f64, 140.0_f64, 181.0_f64);
    let amp = (1.0 - index as f64 / 110.0).max(0.0);
    let beats = match index {
        0..=29 => 4,
        30..=59 => 3,
        60..=79 => 2,
        80..=89 => 1,
        _ => 0,
    };
    let mut d = format!("M {x0},{base}");
    let mut x = x0;
    for _ in 0..beats {
        let a = 10.0 * amp;
        x += w / 24.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base)));
        x += w / 48.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base - a * 0.4)));
        x += w / 48.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base - a)));
        x += w / 48.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base + a * 0.5)));
        x += w / 48.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base)));
        x += w / 16.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base)));
    }
    while x < x0 + w {
        x += w / 12.0;
        d.push_str(&format!(" L {},{}", fmt(x), fmt(base)));
    }
    let color = if index >= 80 { p.blood } else { p.phos };
    let glow = if index >= 80 {
        " filter=\"url(#glow-blood)\""
    } else {
        " filter=\"url(#glow-phos)\""
    };
    let mut s = format!(
        "<path d=\"{d}\" stroke=\"{c}\" stroke-width=\"1.4\" fill=\"none\" stroke-dasharray=\"900\" stroke-dashoffset=\"0\" opacity=\"0.9\"{g}><animate attributeName=\"stroke-dashoffset\" from=\"900\" to=\"0\" dur=\"1.6s\" fill=\"freeze\"/></path>",
        d = d, c = color, g = glow
    );
    if index >= 80 {
        s.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" font-size=\"10\" fill=\"{c}\" class=\"font-mono\"{g}>☠</text>",
            x = fmt(x0 + w + 1.0), y = fmt(base + 3.0), c = color, g = glow
        ));
    } else if index < 15 {
        s.push_str(&format!(
            "<g transform=\"translate({x},{y}) scale(0.85)\"><path d=\"M5,8.5 C2,6 0,4.4 0,2.8 C0,1.2 1.2,0 2.6,0 C3.8,0 4.6,0.7 5,1.5 C5.4,0.7 6.2,0 7.4,0 C8.8,0 10,1.2 10,2.8 C10,4.4 8,6 5,8.5 Z\" fill=\"{c}\"{g}><animate attributeName=\"opacity\" values=\"1;0.4;1\" dur=\"1.2s\" repeatCount=\"indefinite\"/></path></g>",
            x = fmt(x0 + w - 4.0), y = fmt(base - 7.0), c = p.blood, g = glow
        ));
    } else {
        s.push_str(&format!(
            "<circle cx=\"{x}\" cy=\"{y}\" r=\"1.6\" fill=\"{c}\"{g}/>",
            x = fmt(x0 + w),
            y = fmt(base),
            c = color,
            g = glow
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatlines_at_100() {
        let p = super::super::palette::palette(100);
        let svg = render(100, &p);
        assert!(svg.contains("☠"));
    }

    #[test]
    fn heart_at_zero() {
        let p = super::super::palette::palette(0);
        let svg = render(0, &p);
        assert!(svg.contains("M5,8.5"));
    }
}
