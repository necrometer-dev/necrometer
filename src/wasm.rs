//! wasm-bindgen exports — the browser path. JS fetches the repo list
//! (fetch + CORS is native there), hands us JSON; the necromancy is all Rust.

use wasm_bindgen::prelude::*;

use crate::github::Repo;
use crate::metrics::{analyze, Reading, SubjectKind};

/// repos_json: raw JSON array from api.github.com/.../repos.
/// Returns the reading as JSON (camelCase, matches the old JS engine's shape).
#[wasm_bindgen]
pub fn analyze_repos(subject: &str, repos_json: &str) -> Result<String, JsValue> {
    let repos: Vec<Repo> = serde_json::from_str(repos_json)
        .map_err(|e| JsValue::from_str(&format!("repo json: {e}")))?;
    let reading = analyze(subject, SubjectKind::User, &repos);
    serde_json::to_string(&reading).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// reading_json: a Reading JSON (from analyze_repos, possibly round-tripped).
/// Returns the SVG card.
#[wasm_bindgen]
pub fn render_card(reading_json: &str) -> Result<String, JsValue> {
    let reading: Reading = serde_json::from_str(reading_json)
        .map_err(|e| JsValue::from_str(&format!("reading json: {e}")))?;
    Ok(crate::card::render(&reading))
}
