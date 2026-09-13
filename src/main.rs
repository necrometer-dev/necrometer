//! Necrometer daemon entry point.

use anyhow::Result;
use tracing::info;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    necrometer::init_tracing();

    let bind = std::env::var("NECROMETER_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    if std::env::var("GITHUB_TOKEN").is_err() {
        tracing::warn!("GITHUB_TOKEN not set — unauthenticated API rate limit is 60/hr");
    }

    info!(%bind, "necrometer listening");
    axum::serve(listener, necrometer::web::router()?).await?;
    Ok(())
}
