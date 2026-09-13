//! SVG necrometer gauge card renderer — the cute-death pass.
//! Gauge + stats + graveyard strip + skull buddy mascot. Single source of
//! truth: compiled natively for the CLI/service and to wasm for the site.

use crate::metrics::{Fate, Reading};

const W: i32 = 495;
const H: i32 = 195;
const FONT: &str = "-apple-system,'Segoe UI',Roboto,Ubuntu,Cantarell,'Noto Sans',Helvetica,Arial,sans-serif";
/// Cute display font for titles/flavor — <img> SVGs can't load webfonts, so
/// lean on the cute system fonts (Comic Sans on win, Chalkboard on mac).
const FONT_CUTE: &str = "'Comic Sans MS','Chalkboard SE','Segoe Print',-apple-system,'Segoe UI',Roboto,Ubuntu,Cantarell,'Noto Sans',Helvetica,Arial,sans-serif";

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
    let mut s = String::with_capacity(8192);

    s.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="{FONT}">"#
    ));
    s.push_str(&format!(
        r#"<rect width="{W}" height="{H}" rx="6" fill="{}" stroke="{}" stroke-width="1"/>"#,
        p.bg, p.border
    ));

    s.push_str(&gauge(reading.index, &p));
    s.push_str(&stats(reading, &p));
    s.push_str(&graveyard(reading, &p));
    s.push_str(&skull_buddy(reading.index, &p));
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

    // needle: swings in from the alive end on load — SMIL, works in <img>
    let theta = 180.0 - index as f64 * 1.8;
    let swing = (180.0 - theta).max(30.0); // at least a little drama
    let tip = pts(cx, cy, r - 12.0, theta);
    let base_l = pts(cx, cy, 4.0, theta + 90.0);
    let base_r = pts(cx, cy, 4.0, theta - 90.0);
    g.push_str(&format!(
        r#"<g><animateTransform attributeName="transform" type="rotate" values="-{swing:.1} {cx} {cy};4 {cx} {cy};0 {cx} {cy}" keyTimes="0;0.8;1" dur="1.1s" fill="freeze"/>"#
    ));
    g.push_str(&format!(
        r#"<polygon points="{tip} {base_l} {base_r}" fill="{}"/>"#,
        p.needle
    ));
    g.push_str(&format!(
        r#"<circle cx="{cx}" cy="{cy}" r="5.5" fill="{}" stroke="{}" stroke-width="1.5"/></g>"#,
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
        r#"<text x="{x}" y="32" font-size="15" font-weight="600" fill="{}" font-family="{FONT_CUTE}">{}</text>"#,
        p.text,
        esc(&format!("@{}", reading.subject))
    ));

    if reading.total == 0 {
        s.push_str(&format!(
            r#"<text x="{x}" y="60" font-size="13" fill="{}" font-family="{FONT_CUTE}">no repos — nothing to bury</text>"#,
            p.dim
        ));
        return s;
    }

    s.push_str(&format!(
        r#"<text x="{x}" y="70" font-size="34" font-weight="700" fill="{}">{}%</text>"#,
        p.accent, reading.index
    ));
    s.push_str(&format!(
        r#"<text x="{}" y="70" font-size="12" fill="{}">necrotic</text>"#,
        x + 78.0,
        p.dim
    ));
    s.push_str(&format!(
        r#"<text x="{x}" y="90" font-size="13" font-weight="600" fill="{}" font-family="{FONT_CUTE}">{}</text>"#,
        p.accent,
        esc(&reading.title)
    ));
    s.push_str(&format!(
        r#"<text x="{x}" y="105" font-size="10.5" font-style="italic" fill="{}" font-family="{FONT_CUTE}">{}</text>"#,
        p.dim,
        esc(&reading.flavor)
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
        r#"<text x="{x}" y="126" font-size="11" fill="{}">{}</text>"#,
        p.text,
        esc(&fate_line)
    ));

    let mut y = 143.0;
    if reading.stars_stranded > 0 {
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">{} stars stranded on dead repos</text>"#,
            p.dim, reading.stars_stranded
        ));
        y += 15.0;
    }
    if let Some((name, days)) = &reading.oldest_corpse {
        let name = if name.chars().count() > 20 {
            format!("{}…", name.chars().take(19).collect::<String>())
        } else {
            name.clone()
        };
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">oldest corpse: {} ({}d)</text>"#,
            p.dim,
            esc(&name),
            days
        ));
        y += 15.0;
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
        s.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-size="11" fill="{}">{}</text>"#,
            p.dim,
            esc(&bits.join(" · "))
        ));
    }

    s.push_str(&ekg(reading.index, p));
    s
}

/// Heartbeat strip under the stats — spiky when healthy, flatlines when dead.
/// Ends before the watermark (x≈412-485). Send-off: heart healthy, skull dead.
fn ekg(index: u8, p: &Palette) -> String {
    let (x0, w, base) = (200.0_f64, 200.0_f64, 184.0_f64);
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
        let a = 12.0 * amp;
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
    let color = if index >= 80 { "#f85149" } else { &p.accent };
    let mut s = format!(
        r#"<path d="{d}" stroke="{color}" stroke-width="1.4" fill="none" stroke-dasharray="900" stroke-dashoffset="0" opacity="0.85"><animate attributeName="stroke-dashoffset" from="900" to="0" dur="1.6s" fill="freeze"/></path>"#
    );
    if index >= 80 {
        s.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="8" fill="{color}">☠</text>"#,
            fmt(x0 + w + 1.0),
            fmt(base + 3.0)
        ));
    } else if index < 15 {
        s.push_str(&format!(
            r##"<g transform="translate({},{}) scale(0.85)"><path d="M5,8.5 C2,6 0,4.4 0,2.8 C0,1.2 1.2,0 2.6,0 C3.8,0 4.6,0.7 5,1.5 C5.4,0.7 6.2,0 7.4,0 C8.8,0 10,1.2 10,2.8 C10,4.4 8,6 5,8.5 Z" fill="#f778ba"><animate attributeName="opacity" values="1;0.5;1" dur="1.2s" repeatCount="indefinite"/></path></g>"##,
            fmt(x0 + w - 4.0),
            fmt(base - 7.0)
        ));
    } else {
        s.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="1.6" fill="{}"/>"#,
            fmt(x0 + w),
            fmt(base),
            p.dim
        ));
    }
    s
}

/// The mascot: a little skull buddy whose face mirrors the reading.
/// happy = halo + blush + ^^ eyes · ok = dot eyes · sad = frown · dead = x_x + crack
fn skull_buddy(index: u8, p: &Palette) -> String {
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
            r##"<ellipse cx="{sx}" cy="{}" rx="6" ry="2" fill="none" stroke="#ffd866" stroke-width="1.4"/>"##,
            fmt(sy - 16.0)
        ));
    }
    s.push_str(&format!(
        r##"<path d="M {},{} A 10.5 10.5 0 1 1 {},{} L {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z" fill="#e8e6df"/>"##,
        fmt(sx - 10.5), fmt(sy + 3.0),
        fmt(sx + 10.5), fmt(sy + 3.0),
        fmt(sx + 10.5), fmt(sy + 6.0),
        fmt(sx + 10.5), fmt(sy + 10.5), fmt(sx + 6.5), fmt(sy + 10.5),
        fmt(sx - 6.5), fmt(sy + 10.5),
        fmt(sx - 10.5), fmt(sy + 10.5), fmt(sx - 10.5), fmt(sy + 6.0),
    ));
    s.push_str(&format!(
        r#"<path d="M {},{} v -3 M {},{} v -3 M {},{} v -3" stroke="{}" stroke-width="1"/>"#,
        fmt(sx - 3.5), fmt(sy + 10.5),
        fmt(sx), fmt(sy + 10.5),
        fmt(sx + 3.5), fmt(sy + 10.5),
        p.bg
    ));
    let ey = sy - 0.5;
    let eye = |ex: f64| match face {
        "happy" => format!(
            r#"<path d="M {},{} Q {},{} {},{} " stroke="{}" stroke-width="1.5" fill="none" stroke-linecap="round"/>"#,
            fmt(ex - 2.4), fmt(ey + 1.0), fmt(ex), fmt(ey - 2.2), fmt(ex + 2.4), fmt(ey + 1.0), p.bg
        ),
        "dead" => format!(
            r#"<path d="M {},{} l 4,4 M {},{} l -4,4" stroke="{}" stroke-width="1.3" stroke-linecap="round"/>"#,
            fmt(ex - 2.0), fmt(ey - 2.0), fmt(ex + 2.0), fmt(ey - 2.0), p.bg
        ),
        _ => format!(r#"<circle cx="{}" cy="{}" r="2.3" fill="{}"/>"#, fmt(ex), fmt(ey), p.bg),
    };
    s.push_str(&eye(sx - 4.0));
    s.push_str(&eye(sx + 4.0));
    s.push_str(&format!(
        r#"<path d="M {},{} l -1.4,-2.2 l 2.8,0 Z" fill="{}"/>"#,
        fmt(sx), fmt(sy + 4.5), p.bg
    ));
    if face == "sad" {
        s.push_str(&format!(
            r#"<path d="M {},{} Q {},{} {},{}" stroke="{}" stroke-width="1" fill="none"/>"#,
            fmt(sx - 2.5), fmt(sy + 8.4), fmt(sx), fmt(sy + 6.8), fmt(sx + 2.5), fmt(sy + 8.4), p.bg
        ));
    }
    if face == "happy" {
        s.push_str(&format!(
            r##"<ellipse cx="{}" cy="{}" rx="1.8" ry="1.1" fill="#f778ba" opacity="0.7"/><ellipse cx="{}" cy="{}" rx="1.8" ry="1.1" fill="#f778ba" opacity="0.7"/>"##,
            fmt(sx - 6.8), fmt(sy + 4.0), fmt(sx + 6.8), fmt(sy + 4.0)
        ));
    }
    if face == "dead" {
        s.push_str(&format!(
            r##"<path d="M {},{} l 3,3.5 l -1.8,2.5" stroke="#9a978c" stroke-width="0.9" fill="none"/>"##,
            fmt(sx - 4.0), fmt(sy - 8.5)
        ));
    }
    format!("<g>{s}</g>")
}

/// Tiny graveyard under the gauge — a stone per corpse (half-size for
/// stillborn, the baby graves). All alive? Flowers grow instead.
fn graveyard(reading: &Reading, p: &Palette) -> String {
    let gy = 180.0_f64;
    let mut s = format!(
        r#"<line x1="24" y1="{}" x2="166" y2="{}" stroke="{}" stroke-width="1"/>"#,
        fmt(gy), fmt(gy), p.border
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
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#3fb950" stroke-width="1.2"/>"##,
                fmt(fx), fmt(gy), fmt(fx), fmt(gy - 7.0)
            ));
            for k in 0..5 {
                let (px, py) = pt(fx, gy - 9.5, 2.4, k as f64 * 72.0 + 90.0);
                let color = if i % 2 == 1 { "#f778ba" } else { "#ffd866" };
                s.push_str(&format!(
                    r##"<circle cx="{}" cy="{}" r="1.7" fill="{color}"/>"##,
                    fmt(px), fmt(py)
                ));
            }
            s.push_str(&format!(
                r##"<circle cx="{}" cy="{}" r="1.4" fill="#e6edf3"/>"##,
                fmt(fx), fmt(gy - 9.5)
            ));
        }
        return format!("<g>{s}</g>");
    }
    for (i, e) in dead.iter().take(6).enumerate() {
        let (w, h) = if e.stillborn { (8.0, 7.0) } else { (12.0, 11.0) };
        let x = 28.0 + i as f64 * 22.0;
        let top = gy - h;
        s.push_str(&format!(
            r##"<path d="M {},{} L {},{} A {} {} 0 0 1 {},{} L {},{} Z" fill="#565c66"/>"##,
            fmt(x), fmt(gy),
            fmt(x), fmt(top + w / 2.0),
            w / 2.0, w / 2.0,
            fmt(x + w), fmt(top + w / 2.0),
            fmt(x + w), fmt(gy),
        ));
        s.push_str(&format!(
            r#"<path d="M {},{} h 3.6 M {},{} v 3.6" stroke="{}" stroke-width="0.9"/>"#,
            fmt(x + w / 2.0 - 1.8), fmt(top + w / 2.0 + 1.2),
            fmt(x + w / 2.0), fmt(top + w / 2.0 - 0.6),
            p.bg
        ));
    }
    if dead.len() > 6 {
        s.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="8" fill="{}">+{}</text>"#,
            fmt(28.0 + 6.0 * 22.0 - 6.0),
            fmt(gy - 2.0),
            p.dim,
            dead.len() - 6
        ));
    }
    format!("<g>{s}</g>")
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
        // cracks across the dial + cobweb in the corner + its little spider
        let mut e = String::new();
        e.push_str(r##"<polyline points="60,75 72,95 66,110 80,128" stroke="#3a3f45" stroke-width="1.5" fill="none"/>"##);
        e.push_str(r##"<polyline points="120,70 112,92 124,105 115,126" stroke="#3a3f45" stroke-width="1.2" fill="none"/>"##);
        e.push_str(r##"<path d="M 470,10 Q 480,20 488,12 M 470,10 Q 478,28 470,36 M 470,10 L 488,36 M 470,10 A 24 24 0 0 1 488,36" stroke="#3a3f45" stroke-width="1" fill="none"/>"##);
        e.push_str(r##"<line x1="479" y1="24" x2="479" y2="40" stroke="#4a5058" stroke-width="0.8"/>"##);
        e.push_str(r##"<circle cx="479" cy="42" r="2.2" fill="#4a5058"/>"##);
        e.push_str(r##"<path d="M 477,41 l -2.5,-2 M 477,43 l -2.5,2 M 481,41 l 2.5,-2 M 481,43 l 2.5,2" stroke="#4a5058" stroke-width="0.8" fill="none"/>"##);
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
