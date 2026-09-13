//! SVG necrometer gauge card renderer — gothic CRT visual redesign.
//! Gauge + stats + graveyard strip + skull buddy mascot. Single source of
//! truth: compiled natively for the CLI/service and to wasm for the site.

use crate::metrics::{Fate, Reading};

const W: i32 = 495;
const H: i32 = 195;
const FONT: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Ubuntu, Cantarell, 'Noto Sans', Helvetica, Arial, sans-serif";
const FONT_CREEPSTER: &str = "'Creepster', cursive, -apple-system, BlinkMacSystemFont, sans-serif";
const FONT_MONO: &str = "'VT323', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const FONT_SERIF: &str = "'IM Fell English', Georgia, 'Times New Roman', serif";

const CREEPSTER_WOFF2_B64: &str = include_str!("creepster.woff2.b64");

struct Palette {
    bg: &'static str,
    panel: &'static str,
    inset: &'static str,
    border: &'static str,
    text: &'static str,
    dim: &'static str,
    violet: &'static str,
    phos: &'static str,
    blood: &'static str,
    candle: &'static str,
    accent: String,
    needle: String,
    zones: [String; 5],
}

pub fn render(reading: &Reading) -> String {
    let p = palette(reading.index);
    let mut s = String::with_capacity(48000);

    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="{FONT}">"##
    ));
    s.push_str("<defs><style>");
    s.push_str(&format!(
        r##"@font-face {{
  font-family: 'Creepster';
  font-style: normal;
  font-weight: 400;
  src: url(data:font/woff2;charset=utf-8;base64,{}) format('woff2');
}}
.font-creepster {{ font-family: {FONT_CREEPSTER}; }}
.font-mono {{ font-family: {FONT_MONO}; }}
.font-serif {{ font-family: {FONT_SERIF}; }}
.crt-scanlines {{ fill: url(#scanlines); }}
"##,
        CREEPSTER_WOFF2_B64.trim()
    ));
    s.push_str("</style>");
    s.push_str(
        r##"<pattern id="scanlines" width="100%" height="3" patternUnits="userSpaceOnUse"><line x1="0" y1="0" x2="100%" y2="0" stroke="#000000" stroke-width="1" opacity="0.22"/></pattern>"##
    );
    s.push_str(
        r##"<radialGradient id="crt-vignette" cx="50%" cy="45%" r="65%"><stop offset="65%" stop-color="#000000" stop-opacity="0"/><stop offset="100%" stop-color="#000000" stop-opacity="0.55"/></radialGradient>"##
    );
    s.push_str(
        r##"<radialGradient id="screen-glow" cx="50%" cy="35%" r="70%"><stop offset="0%" stop-color="#19122a" stop-opacity="0.6"/><stop offset="100%" stop-color="#0c0a16" stop-opacity="1"/></radialGradient>"##
    );
    s.push_str(
        r##"<filter id="glow-violet" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="0" stdDeviation="2.5" flood-color="#b78cff" flood-opacity="0.55"/></filter>"##
    );
    s.push_str(
        r##"<filter id="glow-phos" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="0" stdDeviation="2.5" flood-color="#7dffa8" flood-opacity="0.55"/></filter>"##
    );
    s.push_str(
        r##"<filter id="glow-blood" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="0" stdDeviation="2.5" flood-color="#ff5470" flood-opacity="0.55"/></filter>"##
    );
    s.push_str(
        r##"<filter id="glow-candle" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="0" stdDeviation="2.5" flood-color="#e8b34b" flood-opacity="0.55"/></filter>"##
    );
    s.push_str("</defs>");

    // Chassis & CRT frame
    s.push_str(&format!(
        r##"<rect width="{W}" height="{H}" rx="10" fill="{}" stroke="{}" stroke-width="1.5"/>"##,
        p.bg, p.border
    ));
    s.push_str(&format!(
        r##"<rect x="5" y="5" width="{}" height="{}" rx="7" fill="{}" stroke="#231c34" stroke-width="1"/>"##,
        W - 10, H - 10, p.panel
    ));
    s.push_str(&format!(
        r##"<rect x="8" y="8" width="{}" height="{}" rx="5" fill="url(#screen-glow)"/>"##,
        W - 16, H - 16
    ));
    s.push_str(&format!(
        r##"<rect x="8" y="8" width="{}" height="{}" rx="5" class="crt-scanlines" pointer-events="none"/>"##,
        W - 16, H - 16
    ));
    s.push_str(&format!(
        r##"<rect x="8" y="8" width="{}" height="{}" rx="5" fill="url(#crt-vignette)" pointer-events="none"/>"##,
        W - 16, H - 16
    ));

    s.push_str(&graveyard(reading, &p));
    s.push_str(&gauge(reading.index, &p));
    s.push_str(&stats(reading, &p));
    s.push_str(&skull_buddy(reading.index, &p));
    s.push_str(&easter_egg(reading.index));

    // Watermark
    s.push_str(&format!(
        r##"<text x="{}" y="{}" text-anchor="end" font-size="10" fill="{}">necrometer.dev</text>"##,
        W - 12,
        H - 10,
        p.dim
    ));
    s.push_str("</svg>");
    s
}

fn palette(index: u8) -> Palette {
    let sat = (100.0 - index as f64).clamp(0.0, 100.0) / 100.0;
    let zones = [
        hsl(140.0, 100.0 * sat, 74.5),
        hsl(101.0, 100.0 * sat, 74.5),
        hsl(40.0, 78.0 * sat, 60.0),
        hsl(20.0, 100.0 * sat, 66.5),
        hsl(350.0, 100.0 * sat, 66.5),
    ];
    let accent = if index >= 80 {
        hsl(350.0, 85.0 * sat, 66.0)
    } else if index >= 50 {
        hsl(38.0, 85.0 * sat, 60.0)
    } else if index >= 20 {
        hsl(263.0, 90.0 * sat, 77.0)
    } else {
        hsl(140.0, 85.0 * sat, 74.5)
    };
    let needle = if index >= 60 {
        "#e8e4d8".into()
    } else {
        hsl(350.0, 85.0 * sat, 66.0)
    };
    Palette {
        bg: "#0a0812",
        panel: "#14101f",
        inset: "#0c0a16",
        border: "#322a4d",
        text: "#e8e4d8",
        dim: "#8a80a8",
        violet: "#b78cff",
        phos: "#7dffa8",
        blood: "#ff5470",
        candle: "#e8b34b",
        accent,
        needle,
        zones,
    }
}

fn gauge(index: u8, p: &Palette) -> String {
    let (cx, cy, r) = (95.0_f64, 132.0_f64, 62.0_f64);
    let mut g = String::new();

    // Inset track groove behind arcs
    g.push_str(
        r##"<path d="M 33.0,132.0 A 62 62 0 0 1 157.0,129.8" stroke="#191428" stroke-width="11" fill="none" stroke-linecap="round"/>"##,
    );

    // Zone arcs: 5 segments of 36deg each, left -> right = vital -> necrotic
    for (i, color) in p.zones.iter().enumerate() {
        let a0 = 180.0 - i as f64 * 36.0;
        let a1 = a0 - 34.0;
        let glow_attr = if i == 0 {
            r#" filter="url(#glow-phos)""#
        } else if i == 4 {
            r#" filter="url(#glow-blood)""#
        } else {
            ""
        };
        g.push_str(&format!(
            r##"<path d="M {} A {} {} 0 0 1 {}" stroke="{}" stroke-width="9" fill="none" stroke-linecap="round"{glow_attr}/>"##,
            pts(cx, cy, r, a0),
            r,
            r,
            pts(cx, cy, r, a1),
            color
        ));
    }

    // Dial face line under the arc
    g.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"##,
        cx - r - 8.0,
        cy,
        cx + r + 8.0,
        cy,
        p.border
    ));

    // End labels
    g.push_str(&format!(
        r##"<text x="{}" y="{}" text-anchor="middle" font-size="11" fill="{}" class="font-mono" filter="url(#glow-phos)">0</text>"##,
        cx - r - 2.0,
        cy + 14.0,
        p.phos
    ));
    g.push_str(&format!(
        r##"<text x="{}" y="{}" text-anchor="middle" font-size="12" fill="{}" class="font-mono" filter="url(#glow-blood)">☠</text>"##,
        cx + r + 2.0,
        cy + 14.0,
        p.blood
    ));

    // Needle: swings in from the alive end on load
    let theta = 180.0 - index as f64 * 1.8;
    let swing = (180.0 - theta).max(30.0);
    let tip = pts(cx, cy, r - 12.0, theta);
    let base_l = pts(cx, cy, 3.8, theta + 90.0);
    let base_r = pts(cx, cy, 3.8, theta - 90.0);
    let needle_glow = if index >= 60 {
        ""
    } else {
        r#" filter="url(#glow-blood)""#
    };
    g.push_str(&format!(
        r##"<g><animateTransform attributeName="transform" type="rotate" values="-{swing:.1} {cx} {cy};4 {cx} {cy};0 {cx} {cy}" keyTimes="0;0.8;1" dur="1.1s" fill="freeze"/>"##
    ));
    g.push_str(&format!(
        r##"<polygon points="{tip} {base_l} {base_r}" fill="{}"{needle_glow}/>"##,
        p.needle
    ));
    g.push_str(&format!(
        r##"<circle cx="{cx}" cy="{cy}" r="5.5" fill="{}" stroke="{}" stroke-width="1.8"/></g>"##,
        p.inset, p.needle
    ));

    g.push_str(&format!(
        r##"<text x="{cx}" y="{}" text-anchor="middle" font-size="14" letter-spacing="2" fill="{}" class="font-creepster" filter="url(#glow-violet)">NECROMETER</text>"##,
        cy + 32.0,
        p.dim
    ));
    g
}

fn stats(reading: &Reading, p: &Palette) -> String {
    let mut s = String::new();
    let x = 200.0;

    s.push_str(&format!(
        r##"<text x="{x}" y="33" font-size="22" letter-spacing="1.5" fill="{}" class="font-creepster" filter="url(#glow-violet)">{}</text>"##,
        p.violet,
        esc(&format!("@{}", reading.subject))
    ));

    if reading.total == 0 {
        s.push_str(&format!(
            r##"<text x="{x}" y="65" font-size="14" fill="{}" class="font-creepster">no repos — nothing to bury</text>"##,
            p.dim
        ));
        return s;
    }

    let idx_glow = if reading.index >= 80 {
        r#" filter="url(#glow-blood)""#
    } else if reading.index >= 50 {
        r#" filter="url(#glow-candle)""#
    } else {
        r#" filter="url(#glow-phos)""#
    };
    s.push_str(&format!(
        r##"<text x="{x}" y="68" font-size="34" font-weight="700" fill="{}" class="font-mono"{idx_glow}>{}%</text>"##,
        p.accent, reading.index
    ));
    let necro_x = if reading.index >= 100 {
        x + 82.0
    } else if reading.index >= 10 {
        x + 70.0
    } else {
        x + 56.0
    };
    s.push_str(&format!(
        r##"<text x="{necro_x}" y="62" font-size="12" letter-spacing="1" fill="{}" class="font-mono">necrotic</text>"##,
        p.dim
    ));
    s.push_str(&format!(
        r##"<text x="{x}" y="88" font-size="16" letter-spacing="1" fill="{}" class="font-creepster" filter="url(#glow-candle)">{}</text>"##,
        p.candle,
        esc(&reading.title)
    ));
    s.push_str(&format!(
        r##"<text x="{x}" y="102" font-size="11" font-style="italic" fill="{}" class="font-serif">{}</text>"##,
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
        r##"<text x="{x}" y="119" font-size="11" fill="{}">{}</text>"##,
        p.text,
        esc(&fate_line)
    ));

    let mut y = 133.0;
    if reading.stars_stranded > 0 {
        s.push_str(&format!(
            r##"<text x="{x}" y="{y}" font-size="10.5" fill="{}">{} stars stranded on dead repos</text>"##,
            p.candle, reading.stars_stranded
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
            r##"<text x="{x}" y="{y}" font-size="10.5" fill="{}">oldest corpse: {} ({}d)</text>"##,
            p.dim,
            esc(&name),
            days
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
            r##"<text x="{x}" y="{y}" font-size="10.5" fill="{col}">{}</text>"##,
            esc(&bits.join(" · "))
        ));
    }

    s.push_str(&ekg(reading.index, p));
    s
}

fn ekg(index: u8, p: &Palette) -> String {
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
        r#" filter="url(#glow-blood)""#
    } else {
        r#" filter="url(#glow-phos)""#
    };
    let mut s = format!(
        r##"<path d="{d}" stroke="{color}" stroke-width="1.4" fill="none" stroke-dasharray="900" stroke-dashoffset="0" opacity="0.9"{glow}><animate attributeName="stroke-dashoffset" from="900" to="0" dur="1.6s" fill="freeze"/></path>"##
    );
    if index >= 80 {
        s.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="10" fill="{color}" class="font-mono"{glow}>☠</text>"##,
            fmt(x0 + w + 1.0),
            fmt(base + 3.0)
        ));
    } else if index < 15 {
        s.push_str(&format!(
            r##"<g transform="translate({},{}) scale(0.85)"><path d="M5,8.5 C2,6 0,4.4 0,2.8 C0,1.2 1.2,0 2.6,0 C3.8,0 4.6,0.7 5,1.5 C5.4,0.7 6.2,0 7.4,0 C8.8,0 10,1.2 10,2.8 C10,4.4 8,6 5,8.5 Z" fill="{}"{glow}><animate attributeName="opacity" values="1;0.4;1" dur="1.2s" repeatCount="indefinite"/></path></g>"##,
            fmt(x0 + w - 4.0),
            fmt(base - 7.0),
            p.blood
        ));
    } else {
        s.push_str(&format!(
            r##"<circle cx="{}" cy="{}" r="1.6" fill="{color}"{glow}/>"##,
            fmt(x0 + w),
            fmt(base)
        ));
    }
    s
}

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
            r##"<ellipse cx="{sx}" cy="{}" rx="6" ry="2" fill="none" stroke="{}" stroke-width="1.5" filter="url(#glow-candle)"/>"##,
            fmt(sy - 17.0),
            p.candle
        ));
    }
    s.push_str(&format!(
        r##"<path d="M {},{} A 10.5 10.5 0 1 1 {},{} L {},{} Q {},{} {},{} L {},{} Q {},{} {},{} Z" fill="{}" stroke="#2d2642" stroke-width="0.8"/>"##,
        fmt(sx - 10.5), fmt(sy + 3.0),
        fmt(sx + 10.5), fmt(sy + 3.0),
        fmt(sx + 10.5), fmt(sy + 6.0),
        fmt(sx + 10.5), fmt(sy + 10.5), fmt(sx + 6.5), fmt(sy + 10.5),
        fmt(sx - 6.5), fmt(sy + 10.5),
        fmt(sx - 10.5), fmt(sy + 10.5), fmt(sx - 10.5), fmt(sy + 6.0),
        p.text
    ));
    s.push_str(&format!(
        r##"<path d="M {},{} v -3 M {},{} v -3 M {},{} v -3" stroke="{}" stroke-width="1"/>"##,
        fmt(sx - 3.5), fmt(sy + 10.5),
        fmt(sx), fmt(sy + 10.5),
        fmt(sx + 3.5), fmt(sy + 10.5),
        p.inset
    ));
    let ey = sy - 0.5;
    let eye = |ex: f64| match face {
        "happy" => format!(
            r##"<path d="M {},{} Q {},{} {},{} " stroke="{}" stroke-width="1.5" fill="none" stroke-linecap="round"/>"##,
            fmt(ex - 2.4), fmt(ey + 1.0), fmt(ex), fmt(ey - 2.2), fmt(ex + 2.4), fmt(ey + 1.0), p.inset
        ),
        "dead" => format!(
            r##"<path d="M {},{} l 4,4 M {},{} l -4,4" stroke="{}" stroke-width="1.3" stroke-linecap="round"/>"##,
            fmt(ex - 2.0), fmt(ey - 2.0), fmt(ex + 2.0), fmt(ey - 2.0), p.inset
        ),
        _ => format!(r##"<circle cx="{}" cy="{}" r="2.3" fill="{}"/>"##, fmt(ex), fmt(ey), p.inset),
    };
    s.push_str(&eye(sx - 4.0));
    s.push_str(&eye(sx + 4.0));
    s.push_str(&format!(
        r##"<path d="M {},{} l -1.4,-2.2 l 2.8,0 Z" fill="{}"/>"##,
        fmt(sx), fmt(sy + 4.5), p.inset
    ));
    if face == "sad" {
        s.push_str(&format!(
            r##"<path d="M {},{} Q {},{} {},{}" stroke="{}" stroke-width="1" fill="none"/>"##,
            fmt(sx - 2.5), fmt(sy + 8.4), fmt(sx), fmt(sy + 6.8), fmt(sx + 2.5), fmt(sy + 8.4), p.inset
        ));
    }
    if face == "happy" {
        s.push_str(&format!(
            r##"<ellipse cx="{}" cy="{}" rx="1.8" ry="1.1" fill="{}" opacity="0.8"/><ellipse cx="{}" cy="{}" rx="1.8" ry="1.1" fill="{}" opacity="0.8"/>"##,
            fmt(sx - 6.8), fmt(sy + 4.0), p.blood,
            fmt(sx + 6.8), fmt(sy + 4.0), p.blood
        ));
    }
    if face == "dead" {
        s.push_str(&format!(
            r##"<path d="M {},{} l 3,3.5 l -1.8,2.5" stroke="{}" stroke-width="0.9" fill="none"/>"##,
            fmt(sx - 4.0), fmt(sy - 8.5), p.dim
        ));
    }
    format!("<g>{s}</g>")
}

fn graveyard(reading: &Reading, p: &Palette) -> String {
    let gy = 180.0_f64;
    let mut s = format!(
        r##"<line x1="24" y1="{}" x2="166" y2="{}" stroke="{}" stroke-width="1"/>"##,
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
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1.2"/>"##,
                fmt(fx), fmt(gy), fmt(fx), fmt(gy - 7.0),
                p.phos
            ));
            for k in 0..5 {
                let (px, py) = pt(fx, gy - 9.5, 2.4, k as f64 * 72.0 + 90.0);
                let color = if i % 2 == 1 { p.blood } else { p.candle };
                s.push_str(&format!(
                    r##"<circle cx="{}" cy="{}" r="1.7" fill="{color}"/>"##,
                    fmt(px), fmt(py)
                ));
            }
            s.push_str(&format!(
                r##"<circle cx="{}" cy="{}" r="1.4" fill="{}"/>"##,
                fmt(fx), fmt(gy - 9.5),
                p.text
            ));
        }
        return format!("<g>{s}</g>");
    }
    for (i, e) in dead.iter().take(6).enumerate() {
        let (w, h) = if e.stillborn { (8.0, 7.0) } else { (12.0, 11.0) };
        let x = 28.0 + i as f64 * 22.0;
        let top = gy - h;
        s.push_str(&format!(
            r##"<path d="M {},{} L {},{} A {} {} 0 0 1 {},{} L {},{} Z" fill="#28213b" stroke="#3d3356" stroke-width="0.8"/>"##,
            fmt(x), fmt(gy),
            fmt(x), fmt(top + w / 2.0),
            w / 2.0, w / 2.0,
            fmt(x + w), fmt(top + w / 2.0),
            fmt(x + w), fmt(gy),
        ));
        s.push_str(&format!(
            r##"<path d="M {},{} h 3.6 M {},{} v 3.6" stroke="{}" stroke-width="0.9"/>"##,
            fmt(x + w / 2.0 - 1.8), fmt(top + w / 2.0 + 1.2),
            fmt(x + w / 2.0), fmt(top + w / 2.0 - 0.6),
            p.inset
        ));
    }
    if dead.len() > 6 {
        s.push_str(&format!(
            r##"<text x="{}" y="{}" font-size="9" fill="{}" class="font-mono">+{}</text>"##,
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
            r##"<circle cx="{cx}" cy="{cy}" r="7" fill="#e8b34b" filter="url(#glow-candle)"/>"##
        ));
        for i in 0..8 {
            let a = i as f64 * 45.0;
            let (x0, y0) = pt(cx, cy, 10.0, a);
            let (x1, y1) = pt(cx, cy, 15.0, a);
            e.push_str(&format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e8b34b" stroke-width="1.5" filter="url(#glow-candle)"/>"##,
                fmt(x0), fmt(y0), fmt(x1), fmt(y1)
            ));
        }
        e
    } else if index >= 80 {
        // cracks across the dial + cobweb in the corner + its little spider
        let mut e = String::new();
        e.push_str(r##"<polyline points="60,75 72,95 66,110 80,128" stroke="#322a4d" stroke-width="1.5" fill="none"/>"##);
        e.push_str(r##"<polyline points="120,70 112,92 124,105 115,126" stroke="#322a4d" stroke-width="1.2" fill="none"/>"##);
        e.push_str(r##"<path d="M 470,10 Q 480,20 488,12 M 470,10 Q 478,28 470,36 M 470,10 L 488,36 M 470,10 A 24 24 0 0 1 488,36" stroke="#322a4d" stroke-width="1" fill="none"/>"##);
        e.push_str(r##"<line x1="479" y1="24" x2="479" y2="40" stroke="#4d4570" stroke-width="0.8"/>"##);
        e.push_str(r##"<circle cx="479" cy="42" r="2.2" fill="#4d4570"/>"##);
        e.push_str(r##"<path d="M 477,41 l -2.5,-2 M 477,43 l -2.5,2 M 481,41 l 2.5,-2 M 481,43 l 2.5,2" stroke="#4d4570" stroke-width="0.8" fill="none"/>"##);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{Reading, SubjectKind};

    fn sample_reading(index: u8, total: u32) -> Reading {
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
    fn test_render_contains_creepster_font() {
        let r = sample_reading(10, 5);
        let svg = render(&r);
        assert!(svg.contains("@font-face"));
        assert!(svg.contains("Creepster"));
        assert!(svg.contains("format('woff2')"));
        assert!(svg.contains(&format!("@{}", r.subject)));
        assert!(svg.contains(&r.title));
        assert!(svg.contains("necrometer.dev"));
    }

    #[test]
    fn test_render_crt_palette() {
        let r = sample_reading(0, 5);
        let svg = render(&r);
        assert!(svg.contains("#0a0812")); // bg
        assert!(svg.contains("#14101f")); // panel
        assert!(svg.contains("#0c0a16")); // inset
        assert!(svg.contains("#322a4d")); // border
        assert!(svg.contains("glow-violet"));
        assert!(svg.contains("scanlines"));
    }

    #[test]
    fn test_render_zero_repos_empty() {
        let mut r = sample_reading(0, 0);
        r.counts = [0; 5];
        let svg = render(&r);
        assert!(svg.contains("no repos — nothing to bury"));
    }

    #[test]
    fn test_render_easter_eggs() {
        let r0 = sample_reading(0, 5);
        let svg0 = render(&r0);
        assert!(svg0.contains("glow-candle")); // sun rays filter

        let r90 = sample_reading(90, 5);
        let svg90 = render(&r90);
        assert!(svg90.contains("polyline")); // cracks
    }

    #[test]
    fn test_xml_escaping() {
        let mut r = sample_reading(10, 1);
        r.subject = "foo&<bar>\"baz".into();
        r.title = "A & B".into();
        r.flavor = "1 < 2 > 0".into();
        let svg = render(&r);
        assert!(svg.contains("foo&amp;&lt;bar&gt;&quot;baz"));
        assert!(svg.contains("A &amp; B"));
        assert!(svg.contains("1 &lt; 2 &gt; 0"));
        assert!(!svg.contains("<bar>"));
    }
}
