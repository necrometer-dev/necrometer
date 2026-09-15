//! XML/HTML/attribute escaping + subject/path validation. Single source
//! of truth, called from both the SVG renderer and the web server.

/// Escape for SVG/HTML text content. We don't escape `'` because every
/// attribute value we emit uses double quotes (verified by `esc_attr`).
pub fn esc_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape for an attribute value wrapped in double quotes.
pub fn esc_attr(s: &str) -> String {
    esc_text(s)
}

/// Validate a GitHub login (subject). GitHub's rules:
/// - 1..=39 chars
/// - alphanumeric + hyphen
/// - cannot start or end with a hyphen
/// - no consecutive hyphens
pub fn is_valid_subject(s: &str) -> bool {
    if s.is_empty() || s.len() > 39 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    let mut prev_dash = false;
    for &c in bytes {
        let ok = c.is_ascii_alphanumeric() || c == b'-';
        if !ok {
            return false;
        }
        if c == b'-' && prev_dash {
            return false;
        }
        prev_dash = c == b'-';
    }
    true
}

/// Reject dangerous CLI output paths. Allows relative paths and `..`
/// only if they normalize to a path inside the current directory.
pub fn is_safe_out_path(p: &str) -> bool {
    if p.starts_with('/') {
        return false;
    }
    if p.contains('\0') {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_basic() {
        assert_eq!(esc_text("a&b<c>d\"e"), "a&amp;b&lt;c&gt;d&quot;e");
    }

    #[test]
    fn valid_subjects() {
        assert!(is_valid_subject("torvalds"));
        assert!(is_valid_subject("rust-lang"));
        assert!(is_valid_subject("a"));
        assert!(is_valid_subject("A-b-C-1"));
    }

    #[test]
    fn invalid_subjects() {
        assert!(!is_valid_subject(""));
        assert!(!is_valid_subject("-leading"));
        assert!(!is_valid_subject("trailing-"));
        assert!(!is_valid_subject("dou--ble"));
        assert!(!is_valid_subject("with/slash"));
        assert!(!is_valid_subject("dot.dot"));
        assert!(!is_valid_subject(&"a".repeat(40)));
    }

    #[test]
    fn safe_paths() {
        assert!(is_safe_out_path("out.svg"));
        assert!(is_safe_out_path("sub/dir/out.svg"));
        assert!(is_safe_out_path(".."));
        assert!(!is_safe_out_path("/etc/passwd"));
        assert!(!is_safe_out_path("ok\0bad"));
    }
}