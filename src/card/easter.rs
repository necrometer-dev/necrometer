//! Easter eggs at the extremes: sunshine for index 0, cracks +
//! cobweb + spider for index >= 80.

use super::geometry::{fmt, pt};

pub fn render(index: u8) -> String {
    if index == 0 {
        sunshine()
    } else if index >= 80 {
        cracked()
    } else {
        String::new()
    }
}

fn sunshine() -> String {
    let mut e = String::new();
    let (cx, cy) = (30.0, 30.0);
    e.push_str(&format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"7\" fill=\"#e8b34b\" filter=\"url(#glow-candle)\"/>"
    ));
    for i in 0..8 {
        let a = i as f64 * 45.0;
        let (x0, y0) = pt(cx, cy, 10.0, a);
        let (x1, y1) = pt(cx, cy, 15.0, a);
        e.push_str(&format!(
            "<line x1=\"{a}\" y1=\"{b}\" x2=\"{c}\" y2=\"{d}\" stroke=\"#e8b34b\" stroke-width=\"1.5\" filter=\"url(#glow-candle)\"/>",
            a = fmt(x0), b = fmt(y0), c = fmt(x1), d = fmt(y1)
        ));
    }
    e
}

fn cracked() -> String {
    let mut e = String::new();
    e.push_str("<polyline points=\"60,75 72,95 66,110 80,128\" stroke=\"#322a4d\" stroke-width=\"1.5\" fill=\"none\"/>");
    e.push_str("<polyline points=\"120,70 112,92 124,105 115,126\" stroke=\"#322a4d\" stroke-width=\"1.2\" fill=\"none\"/>");
    e.push_str("<path d=\"M 470,10 Q 480,20 488,12 M 470,10 Q 478,28 470,36 M 470,10 L 488,36 M 470,10 A 24 24 0 0 1 488,36\" stroke=\"#322a4d\" stroke-width=\"1\" fill=\"none\"/>");
    e.push_str(
        "<line x1=\"479\" y1=\"24\" x2=\"479\" y2=\"40\" stroke=\"#4d4570\" stroke-width=\"0.8\"/>",
    );
    e.push_str("<circle cx=\"479\" cy=\"42\" r=\"2.2\" fill=\"#4d4570\"/>");
    e.push_str("<path d=\"M 477,41 l -2.5,-2 M 477,43 l -2.5,2 M 481,41 l 2.5,-2 M 481,43 l 2.5,2\" stroke=\"#4d4570\" stroke-width=\"0.8\" fill=\"none\"/>");
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunshine_at_zero() {
        assert!(render(0).contains("circle"));
    }

    #[test]
    fn cracks_at_80() {
        assert!(render(80).contains("polyline"));
        assert!(render(100).contains("polyline"));
    }

    #[test]
    fn nothing_in_middle() {
        assert_eq!(render(50), "");
    }
}
