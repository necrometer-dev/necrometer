//! wasm-bindgen exports — the browser path. JS fetches the repo list
//! (fetch + CORS is native there), hands us JSON; the necromancy is all Rust.

use wasm_bindgen::prelude::*;

use crate::card;
use crate::github::parse_repos;
use crate::metrics::{analyze, Fate, Reading, SubjectKind};

/// `repos_json`: raw JSON array from `api.github.com/.../repos`.
/// Returns the reading as JSON.
#[wasm_bindgen]
pub fn analyze_repos(subject: &str, repos_json: &str) -> String {
    let repos = match parse_repos(repos_json) {
        Ok(r) => r,
        Err(e) => return format!("err: {e}"),
    };
    let reading = analyze(subject, SubjectKind::User, &repos);
    serialize_reading(&reading)
}

/// `reading_json`: a Reading JSON (from analyze_repos, possibly
/// round-tripped). Returns the SVG card, or "" on parse error.
#[wasm_bindgen]
pub fn render_card(reading_json: &str) -> String {
    let reading = match deserialize_reading(reading_json) {
        Some(r) => r,
        None => return "err: bad reading".into(),
    };
    card::render(&reading)
}

fn serialize_reading(r: &Reading) -> String {
    crate::json::serialize(&reading_to_value(r))
}

fn deserialize_reading(s: &str) -> Option<Reading> {
    let v = crate::json::parse(s).ok()?;
    value_to_reading(&v)
}

// Minimal ad-hoc (de)serialization for Reading — keeps the public
// JSON shape stable for the JS bridge without pulling a framework.
fn reading_to_value(r: &Reading) -> crate::json::Value {
    use crate::json::Value;
    let mut entries = Vec::new();
    for c in &r.entries {
        entries.push(Value::Object(vec![
            ("name".into(), Value::Str(c.name.clone())),
            ("url".into(), Value::Str(c.url.clone())),
            ("daysIdle".into(), Value::Num(c.days_idle as i64)),
            ("stars".into(), Value::Num(c.stars as i64)),
            ("fate".into(), Value::Str(c.fate.label().into())),
            ("stillborn".into(), Value::Bool(c.stillborn)),
        ]));
    }
    let counts = Value::Array(r.counts.iter().map(|n| Value::Num(*n as i64)).collect());
    Value::Object(vec![
        ("subject".into(), Value::Str(r.subject.clone())),
        ("kind".into(), Value::Str(r.kind.prefix().into())),
        ("index".into(), Value::Num(r.index as i64)),
        ("title".into(), Value::Str(r.title.clone())),
        ("flavor".into(), Value::Str(r.flavor.clone())),
        ("counts".into(), counts),
        ("total".into(), Value::Num(r.total as i64)),
        ("starsStranded".into(), Value::Num(r.stars_stranded as i64)),
        ("oldestCorpse".into(), match &r.oldest_corpse {
            Some((n, d)) => Value::Object(vec![
                ("name".into(), Value::Str(n.clone())),
                ("daysIdle".into(), Value::Num(*d as i64)),
            ]),
            None => Value::Null,
        }),
        ("stillborn".into(), Value::Num(r.stillborn as i64)),
        ("daysSinceAnyPush".into(), match r.days_since_any_push {
            Some(d) => Value::Num(d as i64),
            None => Value::Null,
        }),
        ("lowSample".into(), Value::Bool(r.low_sample)),
        ("entries".into(), Value::Array(entries)),
    ])
}

fn value_to_reading(v: &crate::json::Value) -> Option<Reading> {
    use crate::json::Value;
    let o = match v { Value::Object(o) => o, _ => return None };
    let get = |k: &str| o.iter().find(|(k2, _)| k2 == k).map(|(_, v)| v);
    let s = |k: &str| get(k).and_then(|v| v.as_str()).map(String::from);
    let n = |k: &str| get(k).and_then(|v| v.as_i64()).unwrap_or(0);
    let b = |k: &str| get(k).and_then(|v| v.as_bool()).unwrap_or(false);
    let kind = match s("kind").as_deref() {
        Some("org") => SubjectKind::Org,
        _ => SubjectKind::User,
    };
    let mut counts = [0u32; 5];
    if let Some(Value::Array(a)) = get("counts") {
        for (i, x) in a.iter().enumerate().take(5) {
            counts[i] = x.as_i64().unwrap_or(0) as u32;
        }
    }
    let oldest = if let Some(Value::Object(p)) = get("oldestCorpse") {
        let name = p.iter().find(|(k, _)| *k == "name").and_then(|(_, v)| v.as_str()).map(String::from);
        let days = p.iter().find(|(k, _)| *k == "daysIdle").and_then(|(_, v)| v.as_i64()).unwrap_or(0) as u32;
        name.map(|n| (n, days))
    } else { None };
    let days_since = get("daysSinceAnyPush").and_then(|v| v.as_i64()).map(|d| d as u32);
    let mut entries = Vec::new();
    if let Some(Value::Array(arr)) = get("entries") {
        let now = crate::time::Utc::now();
        for e in arr {
            if let Value::Object(p) = e {
                let name = p.iter().find(|(k,_)|*k=="name").and_then(|(_,v)|v.as_str()).unwrap_or("").to_string();
                let url = p.iter().find(|(k,_)|*k=="url").and_then(|(_,v)|v.as_str()).unwrap_or("").to_string();
                let days_idle = p.iter().find(|(k,_)|*k=="daysIdle").and_then(|(_,v)|v.as_i64()).unwrap_or(0) as u32;
                let stars = p.iter().find(|(k,_)|*k=="stars").and_then(|(_,v)|v.as_i64()).unwrap_or(0) as u64;
                let stillborn = p.iter().find(|(k,_)|*k=="stillborn").and_then(|(_,v)|v.as_bool()).unwrap_or(false);
                let fate = match p.iter().find(|(k,_)|*k=="fate").and_then(|(_,v)|v.as_str()) {
                    Some("alive") => Fate::Alive,
                    Some("cooling") => Fate::Cooling,
                    Some("cold") => Fate::Cold,
                    Some("buried") => Fate::Buried,
                    _ => Fate::Dead,
                };
                entries.push(crate::metrics::Corpse {
                    name, url, days_idle, stars, fate, stillborn,
                    created_at: now,
                    last_activity: now,
                });
            }
        }
    }
    Some(Reading {
        subject: s("subject").unwrap_or_default(),
        kind,
        index: n("index") as u8,
        title: s("title").unwrap_or_default(),
        flavor: s("flavor").unwrap_or_default(),
        counts,
        total: n("total") as u32,
        stars_stranded: n("starsStranded") as u64,
        oldest_corpse: oldest,
        stillborn: n("stillborn") as u32,
        days_since_any_push: days_since,
        low_sample: b("lowSample"),
        entries,
    })
}