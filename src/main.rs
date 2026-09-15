//! CLI: serve | card <user-or-org> [out.svg] | hall <names> [out.json]
//!
//! The default release tarball builds without `--features serve`
//! (card + hall only). The container image builds with
//! `--features serve` to include the daemon.

use std::time::Duration;

use necrometer::error::{Error, Result};
use necrometer::escape::esc_text;
use necrometer::github::routes::{is_valid_subject, validate_out_path, SubjectKind};
use necrometer::metrics::{analyze, Fate};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("card") => card_cmd(&args[1..]),
        Some("hall") => hall_cmd(&args[1..]),
        Some("serve") => serve_cmd(),
        None => serve_cmd(),
        Some(other) => Err(Error::Other(format!(
            "unknown command '{other}' — serve | card <name> [out.svg] | hall <names> [out.json]"
        ))),
    }
}

fn serve_cmd() -> Result<()> {
    #[cfg(not(feature = "serve"))]
    {
        Err(Error::Other(
            "the 'serve' subcommand is not compiled in this build. Rebuild with --features serve."
                .into(),
        ))
    }
    #[cfg(feature = "serve")]
    {
        let bind = std::env::var("NECROMETER_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
        if std::env::var("GITHUB_TOKEN").is_err()
            && std::env::var("GH_TOKEN").is_err()
            && std::env::var("NECRO_TOKEN").is_err()
        {
            eprintln!("WARN no GITHUB_TOKEN — unauthenticated API limit is 60/hr");
        }
        eprintln!("INFO  listening on {bind}");
        necrometer::web::run(&bind)
    }
}

fn card_cmd(args: &[String]) -> Result<()> {
    let name = args
        .first()
        .ok_or_else(|| Error::Other("usage: necrometer card <user-or-org> [out.svg]".into()))?;
    if !is_valid_subject(name) {
        return Err(Error::Other(format!("invalid subject {name:?}")));
    }
    let out = args.get(1).map(String::as_str).unwrap_or("necrometer.svg");
    validate_out_path(out)?;
    let gh = necrometer::github::client::GitHub::new()?;
    let repos = gh
        .resolve_repos(name)
        .map_err(|e| Error::Other(format!("{e}")))?;
    let reading = analyze(name, SubjectKind::User, &repos);
    std::fs::write(out, necrometer::render(&reading))?;
    eprintln!(
        "{}: {}% necrotic ({}) — wrote {out}",
        esc_text(&reading.subject),
        reading.index,
        esc_text(&reading.title)
    );
    Ok(())
}

fn hall_cmd(args: &[String]) -> Result<()> {
    let path = args
        .first()
        .ok_or_else(|| Error::Other("usage: necrometer hall <names-file> [out.json]".into()))?;
    let out = args.get(1).map(String::as_str).unwrap_or("hall.json");
    let names: Vec<String> = std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let gh = necrometer::github::client::GitHub::new()?;
    let mut hall: Vec<HallEntry> = Vec::new();
    for name in &names {
        match gh.resolve_repos(name) {
            Ok(repos) => {
                let r = analyze(name, SubjectKind::User, &repos);
                let corpses = r.entries.iter().filter(|e| e.fate != Fate::Alive).count() as u32;
                hall.push(HallEntry {
                    name: name.clone(),
                    index: r.index,
                    title: r.title.clone(),
                    corpses,
                    total: r.total,
                    stars_stranded: r.stars_stranded,
                });
                eprintln!("{name}: {}% ({corpses}/{})", r.index, r.total);
            }
            Err(e) => {
                eprintln!("{name}: skipped — {e}");
                std::process::exit(2);
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    hall.sort_by(|a, b| b.index.cmp(&a.index).then(b.corpses.cmp(&a.corpses)));
    let mut json = String::from("[");
    for (i, h) in hall.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"name":"{}","index":{},"title":"{}","corpses":{},"total":{},"starsStranded":{}}}"#,
            esc_text(&h.name),
            h.index,
            esc_text(&h.title),
            h.corpses,
            h.total,
            h.stars_stranded
        ));
    }
    json.push_str("]\n");
    std::fs::write(out, json)?;
    eprintln!("wrote {out} ({} entries)", hall.len());
    Ok(())
}

struct HallEntry {
    name: String,
    index: u8,
    title: String,
    corpses: u32,
    total: u32,
    stars_stranded: u64,
}
