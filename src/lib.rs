//! Necrometer core: GitHub necro-metrics, SVG gauge rendering, and
//! the web service. `metrics` and `card` compile to wasm; `github`
//! and `web` are native-only.

pub mod card;
pub mod error;
pub mod escape;
pub mod github;
pub mod json;
pub mod metrics;
pub mod time;

#[cfg(not(target_arch = "wasm32"))]
pub mod http;
#[cfg(not(target_arch = "wasm32"))]
pub mod http_server;

#[cfg(not(target_arch = "wasm32"))]
pub mod web;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export the most common items at the crate root for ergonomics.
pub use metrics::{analyze, Fate, Reading, SubjectKind};
pub use card::render;
#[cfg(not(target_arch = "wasm32"))]
pub use github::client::{FetchError, GitHub};
#[cfg(not(target_arch = "wasm32"))]
pub use github::Repo;