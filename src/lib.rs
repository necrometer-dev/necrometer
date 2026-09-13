//! Necrometer core — GitHub necro-metrics, SVG gauge rendering, and the web service.
//! metrics/card compile to wasm for the site; github/web are native-only.

pub mod card;
pub mod github; // Repo is wasm-safe; the reqwest client is cfg-gated inside
pub mod metrics;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(not(target_arch = "wasm32"))]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}
