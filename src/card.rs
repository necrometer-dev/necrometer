//! SVG necrometer gauge card renderer.

use crate::metrics::Reading;

const W: i32 = 495;
const H: i32 = 195;
const FONT: &str = "-apple-system,'Segoe UI',Roboto,Ubuntu,Cantarell,'Noto Sans',Helvetica,Arial,sans-serif";

struct Palette {
    bg: &'static str,
    border: &'static str,
    text: &'static str,
    dim: &'static str,
    accent: String,
    needle: String,
    zones: [String; 5],
}

pub fn render(reading: &Reading) -> String {
    let p = palette(reading.index);
    let mut s = String::with_capacity(4096);

    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="{FONT}">"#
    ));
    s.push_str(&format!(
        r#"<rect width="{W}" height="{H}" rx="6" fill="{}" stroke="{}" stroke-width="1"/>"#,
        p.bg, p.border
    ));

    s.push_str(&gauge(reading.index, &p));
    s.push_str(&stats(reading, &p));
    s.push_str(&easter_egg(reading.index));

    s.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="end" font-size="10" fill="{}">necrometer.dev</text>"#,
        W - 10,
        H - 8,
        p.dim
    ));
    s.push_str("</svg>");
    s
}

fn palette(index: u8) -> Palette {
    let sat = (100.0 - index as f64).clamp(0.0, 100.0) / 100.0; // saturation factor 0..1
    let zones = [
        hsl(150.0, 70.0 * sat, 48.0),
        hsl(95.0, 65.0 * sat, 50.0),
        hsl(45.0, 85.0 * sat, 55.0),
        hsl(20.0, 85.0 * sat, 55.0),
        hsl(0.0, 75.0 * sat, 52.0),
    ];
    Palette {
        bg: "#0d1117",
        border: "#30363d",
        text: "#e6edf3",
        dim: "#8b949e",
        accent: hsl(275.0, 75.0 * sat, 62.0),
        needle: if index >= 60 {
            "#d8d4c8".into() // bone needle for the dead
        } else {
            hsl(350.0, 80.0 * sat + 10.0, 58.0)
        },
        zones,
    }
}

fn gauge(index: u8, p: &Palette) -> String {
    let (cx, cy, r) = (95.0_f64, 132.0_f64, 62.0_f64);
    let mut g = String::new();

    // zone arcs: 5 segments of 36deg each, left -> right = vital -> necrotic
    for (i, color) in p.zones.iter().enumerate() {
        let a0 = 180.0 - i as f64 * 36.0;
        let a1 = a0 - 34.0; // 2deg gap between zones
        g.push_str(&format!(
            r#"<path d="M {} A {} {} 0 0 1 {}" stroke="{}" stroke-width="9" fill="none" stroke-linecap="round"/>"#,
            pts(cx, cy, r, a0),
            r,
            r,
            pts(cx, cy, r, a1),
            color
        ));
    }

    // dial face line under the arc
    g.push_str(&format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"#,
        cx - r - 8.0,
        cy,
        cx + r + 8.0,
        cy,
        p.border
    ));

    // end labels
    g.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="middle" font-size="9" fill="{}">0</text>"#,
        cx - r - 2.0,
        cy + 14.0,
        p.dim
    ));
    g.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="middle" font-size="9" fill="{}">☠</text>"#,
        cx + r + 2.0,
        cy + 14.0,
        p.dim
    ));

    // needle: index 0 -> far left (vital), 100 -> far right (necrotic)
    let theta = 180.0 - index as f64 * 1.8;
    let tip = pts(cx, cy, r - 12.0, theta);
    let base_l = pts(cx, cy, 4.0, theta + 90.0);
    let base_r = pts(cx, cy, 4.0, theta - 90.0);
    g.push_str(&format!(
        r#"<polygon points="{tip} {base_l} {base_r}" fill="{}"/>"#,
        p.needle
    ));
    g.push_str(&format!(
        r#"<circle cx="{cx}" cy="{cy}" r="5.5" fill="{}" stroke="{}" stroke-width="1.5"/>"#,
        p.bg, p.needle
    ));

    g.push_str(&format!(
        r#"<text x="{cx}" y="{}" text-anchor="middle" font-size="10" letter-spacing="2" fill="{}">NECROMETER</text>"#,
        cy + 32.0,
        p.dim
    ));
    g
}

fn stats(reading: &Reading, p: &Palette) -> String {
    let mut s = String::new();
    let x = 200.0;

    s.push_str(&format!(
        r#"<text x="{x}" y="32" font-size="15" font-weight="600" fill="{}">{}</text>"#,
        p.text,
        esc(&format!("@{}", reading.subject))
    ));

    if reading.total == 0 {
        s.push_str(&format!(
            r#"<text x="{x}" y="60" font-size="13" fill="{}">no repos — nothing to bury</text>"#,
            p.dim
        ));
        return s;
    }

    s.push_str(&format!(
        r#"<text x="{x}" y="72" font-size="34" font-weight="700" fill="{}">{}%</text>"#,
        p.accent, reading.index
    ));
    s.push_str(&format!(
        r#"<text x="{}" y="72" font-size="12" fill="{}">necrotic</text>"#,
        x + 78.0,
        p.dim
    ));
    s.push_str(&format!(
        r#"<text x="{x}" y="94" font-size="13" font-style="italic" fill="{}">{}</text>"#,
        p.accent,
        esc(&reading.title)
    ));

    let fate_line = [
        ("alive", reading.counts[0]),
        ("cooling", reading.counts[1]),
        ("cold", reading.counts[2]),
        ("dead", reading.counts[3]),
        ("buried", reading.counts[4]),
    ]
    .iter()
    .filter(|(_, n)| *n > 0)
    .map(|(l, n)| format!("{n} {l}"))
    .collect::<Vec<_>>()
    .join(" · ");
    s.push_str(&format!(
        r#"<text x="{x}" y="122" font-size="11" fill="{}">{}</text>"#,
        p.text,
        esc(&fate_line)
    ));

    let mut y = 142.0;
    if reading.stars_stranded > 0 {
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">{} stars stranded on dead repos</text>"#,
            p.dim, reading.stars_stranded
        ));
        y += 16.0;
    }
    if let Some((name, days)) = &reading.oldest_corpse {
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">oldest corpse: {} ({}d)</text>"#,
            p.dim,
            esc(name),
            days
        ));
        y += 16.0;
    }
    if let Some(d) = reading.days_since_any_push {
        let msg = if d == 0 {
            "signs of life today".to_string()
        } else {
            format!("last sign of life: {d}d ago")
        };
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">{}</text>"#,
            p.dim, msg
        ));
    }
    if reading.stillborn > 0 {
        s.push_str(&format!(
            r#"<text x="{x}" y="{}" font-size="11" fill="{}">{} stillborn repos</text>"#,
            y + 16.0,
            p.dim,
            reading.stillborn
        ));
    }
    s
}

fn easter_egg(index: u8) -> String {
    if index == 0 {
        // sunshine: rays around a small sun, upper-left corner
        let mut e = String::new();
        let (cx, cy) = (30.0, 30.0);
        e.push_str(&format!(
            r##"<circle cx="{cx}" cy="{cy}" r="7" fill="#ffd866"/>"##
        ));
        for i in 0..8 {
            let a = i as f64 * 45.0;
            let (x0, y0) = pt(cx, cy, 10.0, a);
            let (x1, y1) = pt(cx, cy, 15.0, a);
            e.push_str(&format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#ffd866" stroke-width="1.5"/>"##,
                fmt(x0), fmt(y0), fmt(x1), fmt(y1)
            ));
        }
        e
    } else if index >= 80 {
        // cracks across the dial + cobweb in the corner
        let mut e = String::new();
        e.push_str(r##"<polyline points="60,75 72,95 66,110 80,128" stroke="#3a3f45" stroke-width="1.5" fill="none"/>"##);
        e.push_str(r##"<polyline points="120,70 112,92 124,105 115,126" stroke="#3a3f45" stroke-width="1.2" fill="none"/>"##);
        e.push_str(r##"<path d="M 470,10 Q 480,20 488,12 M 470,10 Q 478,28 470,36 M 470,10 L 488,36 M 470,10 A 24 24 0 0 1 488,36" stroke="#3a3f45" stroke-width="1" fill="none"/>"##);
        e
    } else {
        String::new()
    }
}

fn pt(cx: f64, cy: f64, r: f64, deg: f64) -> (f64, f64) {
    let rad = deg.to_radians();
    (cx + r * rad.cos(), cy - r * rad.sin())
}

fn pts(cx: f64, cy: f64, r: f64, deg: f64) -> String {
    let (x, y) = pt(cx, cy, r, deg);
    format!("{},{}", fmt(x), fmt(y))
}

fn fmt(v: f64) -> String {
    format!("{v:.1}")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn hsl(h: f64, s: f64, l: f64) -> String {
    let s = s.clamp(0.0, 100.0) / 100.0;
    let l = l.clamp(0.0, 100.0) / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (r, g, b) = match hp as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    format!(
        "#{:02x}{:02x}{:02x}",
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8
    )
}
