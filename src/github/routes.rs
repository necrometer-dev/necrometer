//! Single source of truth for validating subjects and CLI output paths.
//! `SubjectKind` is re-exported from `crate::metrics` so there's one
//! definition site.

pub use crate::metrics::SubjectKind;

use crate::error::{Error, Result};

/// Validates a GitHub login. Mirrors GitHub's rules:
/// 1..=39 chars, alnum + hyphen, no leading/trailing hyphen,
/// no consecutive hyphens.
pub fn is_valid_subject(s: &str) -> bool {
    crate::escape::is_valid_subject(s)
}

/// Reject dangerous output paths. The CLI refuses to write anywhere
/// outside the current working directory.
pub fn validate_out_path(p: &str) -> Result<()> {
    if !crate::escape::is_safe_out_path(p) {
        return Err(Error::Other(format!("refusing to write to {p:?}")));
    }
    let canon = std::fs::canonicalize(".").unwrap_or_else(|_| std::path::PathBuf::from("."));
    let joined = canon.join(p);
    let normalized = match std::fs::canonicalize(joined.parent().unwrap_or(&canon)) {
        Ok(p) => p,
        Err(_) => canon.clone(),
    };
    if !normalized.starts_with(&canon) {
        return Err(Error::Other(format!("path escapes cwd: {p:?}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        assert!(is_valid_subject("torvalds"));
        assert!(is_valid_subject("rust-lang"));
        assert!(!is_valid_subject(""));
        assert!(!is_valid_subject("-leading"));
        assert!(!is_valid_subject("trailing-"));
        assert!(!is_valid_subject("dou--ble"));
        assert!(!is_valid_subject("with/slash"));
        assert!(!is_valid_subject(&"a".repeat(40)));
    }
}