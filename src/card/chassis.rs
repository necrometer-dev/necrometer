//! The SVG chassis: opening `<svg>` tag, `<defs>` (gradients,
//! filters, font embedding), and the bezel/CRT frame. Watermark at
//! the bottom-right.

use super::palette::{Palette, CANVAS_H, CANVAS_W, FONT};

pub const CREEPSTER_WOFF2_B64: &str = include_str!("creepster.woff2.b64");

pub fn svg_open() -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" font-family=\"{font}\">",
        w = CANVAS_W, h = CANVAS_H, font = FONT
    )
}

pub fn defs() -> String {
    let mut s = String::from("<defs><style>");
    s.push_str(&format!(
        "@font-face {{ font-family: 'Creepster'; font-style: normal; font-weight: 400; src: url(data:font/woff2;charset=utf-8;base64,{}) format('woff2'); }}\n",
        CREEPSTER_WOFF2_B64.trim()
    ));
    s.push_str("</style>");
    s.push_str("<pattern id=\"scanlines\" width=\"100%\" height=\"3\" patternUnits=\"userSpaceOnUse\"><line x1=\"0\" y1=\"0\" x2=\"100%\" y2=\"0\" stroke=\"#000000\" stroke-width=\"1\" opacity=\"0.22\"/></pattern>");
    s.push_str("<radialGradient id=\"crt-vignette\" cx=\"50%\" cy=\"45%\" r=\"65%\"><stop offset=\"65%\" stop-color=\"#000000\" stop-opacity=\"0\"/><stop offset=\"100%\" stop-color=\"#000000\" stop-opacity=\"0.55\"/></radialGradient>");
    s.push_str("<radialGradient id=\"screen-glow\" cx=\"50%\" cy=\"35%\" r=\"70%\"><stop offset=\"0%\" stop-color=\"#19122a\" stop-opacity=\"0.6\"/><stop offset=\"100%\" stop-color=\"#0c0a16\" stop-opacity=\"1\"/></radialGradient>");
    for (id, color) in [
        ("violet", "#b78cff"),
        ("phos", "#7dffa8"),
        ("blood", "#ff5470"),
        ("candle", "#e8b34b"),
    ] {
        s.push_str(&format!(
            "<filter id=\"glow-{id}\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\"><feDropShadow dx=\"0\" dy=\"0\" stdDeviation=\"2.5\" flood-color=\"{color}\" flood-opacity=\"0.55\"/></filter>",
        ));
    }
    s.push_str("</defs>");
    s
}

pub fn chassis(p: &Palette) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "<rect width=\"{w}\" height=\"{h}\" rx=\"10\" fill=\"{bg}\" stroke=\"{border}\" stroke-width=\"1.5\"/>",
        w = CANVAS_W, h = CANVAS_H, bg = p.bg, border = p.border
    ));
    s.push_str(&format!(
        "<rect x=\"5\" y=\"5\" width=\"{w}\" height=\"{h}\" rx=\"7\" fill=\"{panel}\" stroke=\"#231c34\" stroke-width=\"1\"/>",
        w = CANVAS_W - 10, h = CANVAS_H - 10, panel = p.panel
    ));
    for (cls, fill) in [
        ("", "url(#screen-glow)"),
        ("crt-scanlines", ""),
        ("crt-vignette", ""),
    ] {
        let fill_attr = if fill.is_empty() {
            String::new()
        } else {
            format!(" fill=\"{fill}\"")
        };
        let class_attr = if cls.is_empty() {
            String::new()
        } else {
            format!(" class=\"{cls}\"")
        };
        let pe_attr = if cls == "crt-vignette" || cls == "crt-scanlines" {
            " pointer-events=\"none\"".to_string()
        } else {
            String::new()
        };
        s.push_str(&format!(
            "<rect x=\"8\" y=\"8\" width=\"{w}\" height=\"{h}\" rx=\"5\"{fill}{class}{pe}/>",
            w = CANVAS_W - 16,
            h = CANVAS_H - 16,
            fill = fill_attr,
            class = class_attr,
            pe = pe_attr
        ));
    }
    s
}

pub fn watermark(p: &Palette) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" text-anchor=\"end\" font-size=\"10\" fill=\"{c}\">necrometer.dev</text>",
        x = CANVAS_W - 12, y = CANVAS_H - 10, c = p.dim
    )
}
