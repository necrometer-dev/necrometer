//! Necrometer — daemon, card generator, and hall-of-corpses builder.
//!
//!   necrometer                       serve the web service (default)
//!   necrometer card <user-or-org> [out.svg]    fetch + analyze + render a card
//!   necrometer hall <names-file> [out.json]    build hall.json from a list

use anyhow::{bail, Context, Result};
use tracing::info;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    necrometer::init_tracing();

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("card") => card_cmd(&args[1..]).await,
        Some("hall") => hall_cmd(&args[1..]).await,
        Some("serve") | None => serve().await,
        Some(other) => bail!("unknown command '{other}' — serve | card <name> [out.svg] | hall <names> [out.json]"),
    }
}

async fn serve() -> Result<()> {
    let bind = std::env::var("NECROMETER_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;

    if std::env::var("GITHUB_TOKEN").is_err() {
        tracing::warn!("GITHUB_TOKEN not set — unauthenticated API rate limit is 60/hr");
    }

    info!(%bind, "necrometer listening");
    axum::serve(listener, necrometer::web::router()?).await?;
    Ok(())
}

async fn card_cmd(args: &[String]) -> Result<()> {
    let name = args.first().context("usage: necrometer card <user-or-org> [out.svg]")?;
    let out = args.get(1).map(String::as_str).unwrap_or("necrometer.svg");
    let gh = necrometer::github::GitHub::new()?;
    let repos = gh.resolve_repos(name).await?;
    let reading = necrometer::metrics::analyze(name, necrometer::metrics::SubjectKind::User, &repos);
    std::fs::write(out, necrometer::card::render(&reading))?;
    eprintln!(
        "{}: {}% necrotic ({}) — wrote {out}",
        reading.subject, reading.index, reading.title
    );
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HallEntry {
    name: String,
    index: u8,
    title: String,
    corpses: u32,
    total: u32,
    stars_stranded: u64,
}

async fn hall_cmd(args: &[String]) -> Result<()> {
    let path = args.first().context("usage: necrometer hall <names-file> [out.json]")?;
    let out = args.get(1).map(String::as_str).unwrap_or("hall.json");
    let names: Vec<String> = std::fs::read_to_string(path)
        .with_context(|| format!("reading {path}"))?
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();

    let gh = necrometer::github::GitHub::new()?;
    let mut hall = Vec::new();
    for name in &names {
        match gh.resolve_repos(name).await {
            Ok(repos) => {
                let r = necrometer::metrics::analyze(
                    name,
                    necrometer::metrics::SubjectKind::User,
                    &repos,
                );
                hall.push(HallEntry {
                    name: name.clone(),
                    index: r.index,
                    title: r.title.clone(),
                    corpses: r.entries.iter().filter(|e| e.fate != necrometer::metrics::Fate::Alive).count() as u32,
                    total: r.total,
                    stars_stranded: r.stars_stranded,
                });
                eprintln!("{name}: {}% ({}/{})", r.index, hall.last().unwrap().corpses, r.total);
            }
            Err(e) => eprintln!("{name}: skipped — {e}"),
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    hall.sort_by(|a, b| b.index.cmp(&a.index).then(b.corpses.cmp(&a.corpses)));
    std::fs::write(out, serde_json::to_string_pretty(&hall)? + "\n")?;
    eprintln!("wrote {out} ({} entries)", hall.len());
    Ok(())
}
