//! SVG text escaping — re-export from `crate::escape` so card
//! submodules don't reach into the crate root.

pub use crate::escape::{esc_attr, esc_text};
