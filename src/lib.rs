//! Necrometer core — GitHub necro-metrics, SVG gauge rendering, and the web service.

pub mod card;
pub mod github;
pub mod metrics;
pub mod web;

pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();
}
