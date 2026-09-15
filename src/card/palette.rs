//! Color palette and HSL→hex conversion. The visual identity of the
//! card lives here.

const W: i32 = 495;
const H: i32 = 195;
pub const FONT: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Ubuntu, Cantarell, 'Noto Sans', Helvetica, Arial, sans-serif";
pub const FONT_CREEPSTER: &str = "'Creepster', cursive, -apple-system, BlinkMacSystemFont, sans-serif";
pub const FONT_MONO: &str = "'VT323', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
pub const FONT_SERIF: &str = "'IM Fell English', Georgia, 'Times New Roman', serif";
pub const CANVAS_W: i32 = W;
pub const CANVAS_H: i32 = H;

pub struct Palette {
    pub bg: &'static str,
    pub panel: &'static str,
    pub inset: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub dim: &'static str,
    pub violet: &'static str,
    pub phos: &'static str,
    pub blood: &'static str,
    pub candle: &'static str,
    pub accent: String,
    pub needle: String,
    pub zones: [String; 5],
}

pub fn palette(index: u8) -> Palette {
    let sat = ((100.0 - index as f64).clamp(0.0, 100.0)) / 100.0;
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
    let needle = if index >= 60 { "#e8e4d8".into() } else { hsl(350.0, 85.0 * sat, 66.0) };
    Palette {
        bg: "#0a0812", panel: "#14101f", inset: "#0c0a16", border: "#322a4d",
        text: "#e8e4d8", dim: "#8a80a8", violet: "#b78cff", phos: "#7dffa8",
        blood: "#ff5470", candle: "#e8b34b",
        accent, needle, zones,
    }
}

pub fn hsl(h: f64, s: f64, l: f64) -> String {
    let s = s.clamp(0.0, 100.0) / 100.0;
    let l = l.clamp(0.0, 100.0) / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let xm = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (r, g, b) = match hp as i32 {
        0 => (c, xm, 0.0),
        1 => (xm, c, 0.0),
        2 => (0.0, c, xm),
        3 => (0.0, xm, c),
        4 => (xm, 0.0, c),
        _ => (c, 0.0, xm),
    };
    let m = l - c / 2.0;
    format!("#{:02x}{:02x}{:02x}",
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_extremes() {
        let p0 = palette(0);
        let p100 = palette(100);
        assert_eq!(p0.bg, "#0a0812");
        assert!(p100.zones[4].starts_with('#'));
    }

    #[test]
    fn hsl_basic() {
        assert_eq!(hsl(0.0, 100.0, 50.0), "#ff0000");
        assert_eq!(hsl(120.0, 100.0, 50.0), "#00ff00");
        assert_eq!(hsl(240.0, 100.0, 50.0), "#0000ff");
    }
}