//! Minimal time handling. Replaces chrono.
//!
//! `Utc` is a thin wrapper around an i64 epoch second count. We only
//! need: `now()`, parse RFC 3339, and compute day deltas. No timezones,
//! no formatting, no calendars. Hand-rolled — std::time::SystemTime is
//! the only clock.

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(
    inline_js = "export function __necrometer_now() { return Math.floor(Date.now() / 1000); }"
)]
extern "C" {
    fn __necrometer_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
fn wasm_clock_secs() -> Option<i64> {
    Some(__necrometer_now() as i64)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utc(pub i64);

impl Utc {
    pub fn now() -> Self {
        // SystemTime is unavailable on wasm32; fall back to JS Date if a
        // clock was registered, otherwise 0 (the caller decides what to
        // do with "born at the epoch" repos).
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(secs) = wasm_clock_secs() {
                return Utc(secs);
            }
            return Utc(0);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            Utc(secs)
        }
    }

    /// Parse a subset of RFC 3339: `YYYY-MM-DDTHH:MM:SS[Z|±HH:MM]`.
    /// GitHub's repo timestamps are always UTC, with either a trailing
    /// `Z` or `+00:00`. Anything we can't parse returns `None`, and
    /// callers treat that as "born dead" (no push recorded).
    pub fn parse_rfc3339(s: &str) -> Option<Self> {
        let s = s.trim();
        let bytes = s.as_bytes();
        if bytes.len() < 20 {
            return None;
        }
        let year = parse_int(&bytes[0..4])?;
        if bytes[4] != b'-' {
            return None;
        }
        let month = parse_int(&bytes[5..7])?;
        if bytes[7] != b'-' {
            return None;
        }
        let day = parse_int(&bytes[8..10])?;
        if bytes[10] != b'T' && bytes[10] != b' ' {
            return None;
        }
        let hour = parse_int(&bytes[11..13])?;
        if bytes[13] != b':' {
            return None;
        }
        let minute = parse_int(&bytes[14..16])?;
        if bytes[16] != b':' {
            return None;
        }
        let second = parse_int(&bytes[17..19])?;

        // Optional fractional seconds: .NNN — ignore beyond seconds.
        let mut idx = 19;
        if idx < bytes.len() && bytes[idx] == b'.' {
            idx += 1;
            while idx < bytes.len() && bytes[idx].is_ascii_digit() {
                idx += 1;
            }
        }

        // Timezone: Z, +HH:MM, -HH:MM. Everything resolves to UTC.
        let mut tz_offset_secs: i64 = 0;
        if idx < bytes.len() {
            match bytes[idx] {
                b'Z' | b'z' => {}
                b'+' => {
                    idx += 1;
                    let oh = parse_int_range(&bytes[idx..], 2)? as i64;
                    idx += 2;
                    if idx < bytes.len() && bytes[idx] == b':' {
                        idx += 1;
                    }
                    let om = parse_int_range(&bytes[idx..], 2)? as i64;
                    tz_offset_secs = oh * 3600 + om * 60;
                }
                b'-' => {
                    idx += 1;
                    let oh = parse_int_range(&bytes[idx..], 2)? as i64;
                    idx += 2;
                    if idx < bytes.len() && bytes[idx] == b':' {
                        idx += 1;
                    }
                    let om = parse_int_range(&bytes[idx..], 2)? as i64;
                    tz_offset_secs = -(oh * 3600 + om * 60);
                }
                _ => return None,
            }
        }

        let days = days_from_civil(year, month, day)?;
        let secs =
            days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64 - tz_offset_secs;
        Some(Utc(secs))
    }

    pub fn to_rfc3339(self) -> String {
        let secs = self.0;
        let days = secs.div_euclid(86400);
        let rem = secs.rem_euclid(86400) as u32;
        let (year, month, day, _h, _m, _s) = civil_from_days(days);
        let hour = rem / 3600;
        let minute = (rem / 60) % 60;
        let second = rem % 60;
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",)
    }
}

fn parse_int(b: &[u8]) -> Option<u32> {
    if b.len() < 2 {
        return None;
    }
    parse_int_range(b, b.len().min(4))
}

fn parse_int_range(b: &[u8], n: usize) -> Option<u32> {
    if b.len() < n {
        return None;
    }
    let mut v: u32 = 0;
    for &c in &b[..n] {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v * 10 + (c - b'0') as u32;
    }
    Some(v)
}

/// Howard Hinnant's days_from_civil: days since 1970-01-01 for a given
/// proleptic Gregorian date. Returns None for impossible dates.
fn days_from_civil(y: u32, m: u32, d: u32) -> Option<i64> {
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let m = if m <= 2 { m + 9 } else { m - 3 } as i64;
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days)
}

fn civil_from_days(z: i64) -> (u32, u32, u32, u32, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = (if m <= 2 { y + 1 } else { y }) as u32;
    let secs_in_day = z.rem_euclid(86400) as u32;
    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;
    (y, m, d, hour, minute, second)
}

impl std::ops::Sub for Utc {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub i64);

impl Duration {
    pub fn num_days(&self) -> i64 {
        self.0 / 86400
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_z() {
        let t = Utc::parse_rfc3339("2024-01-15T12:34:56Z").unwrap();
        assert!(t.0 > 1_700_000_000 && t.0 < 1_800_000_000);
    }

    #[test]
    fn parse_offset() {
        let a = Utc::parse_rfc3339("2024-01-15T12:34:56Z").unwrap();
        let b = Utc::parse_rfc3339("2024-01-15T13:34:56+01:00").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn roundtrip() {
        let a = Utc::now();
        let s = a.to_rfc3339();
        let b = Utc::parse_rfc3339(&s).unwrap();
        // 60-second bucket because the formatter doesn't include subseconds.
        assert_eq!(a.0 / 60, b.0 / 60);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Utc::parse_rfc3339("nope").is_none());
        assert!(Utc::parse_rfc3339("2024-13-40T99:99:99Z").is_none());
    }
}
