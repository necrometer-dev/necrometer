//! Coordinate helpers: convert (cx, cy, r, deg) → SVG path point.

/// Compute a 2D point on a circle. `deg` is measured clockwise from
/// "up" (12 o'clock), like a clock. Returns (x, y) in SVG coordinates.
pub fn pt(cx: f64, cy: f64, r: f64, deg: f64) -> (f64, f64) {
    let rad = deg.to_radians();
    (cx + r * rad.sin(), cy - r * rad.cos())
}

/// Same as `pt` but formatted as `x,y` for use in SVG `<path d="...">`.
pub fn pts(cx: f64, cy: f64, r: f64, deg: f64) -> String {
    let (x, y) = pt(cx, cy, r, deg);
    format!("{},{}", fmt(x), fmt(y))
}

/// One decimal of precision — readable, identical to the prior output.
pub fn fmt(v: f64) -> String {
    format!("{v:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_up() {
        // 0° points up (north). SVG y grows down, so y = cy - r.
        let (x, y) = pt(0.0, 0.0, 10.0, 0.0);
        assert!((x - 0.0).abs() < 1e-9);
        assert!((y - -10.0).abs() < 1e-9);
    }

    #[test]
    fn pt_right() {
        // 90° points right (east).
        let (x, y) = pt(0.0, 0.0, 10.0, 90.0);
        assert!((x - 10.0).abs() < 1e-9);
        assert!((y - 0.0).abs() < 1e-9);
    }
}
