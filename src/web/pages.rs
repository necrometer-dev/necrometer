//! HTML pages: landing, subject, not-found, upstream-error.

use super::css::CSS;
use crate::escape::esc_text;
use crate::metrics::Reading;

pub fn page(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{t}</title>
<meta property="og:title" content="{t}">
<style>{css}</style>
</head><body><main>{body}</main></body></html>"#,
        t = esc_text(title),
        css = CSS,
        body = body
    )
}

pub fn landing() -> String {
    page(
        "necrometer — how dead are your repos?",
        r#"
<h1>THE NECROMETER</h1>
<p class="tag">Measures how dead your GitHub repos are. Embeds in your README, haunts your profile.</p>
<form action="/go" method="get">
<input name="subject" placeholder="github username or org" autofocus>
<button type="submit">check the pulse</button>
</form>
<p class="hint">embed: <code>[![Necrometer](https://necrometer.dev/u/YOU.svg)](https://necrometer.dev/u/YOU)</code></p>
<h2>specimens</h2>
<img src="/u/torvalds.svg" alt="torvalds necrometer">
<img src="/org/rust-lang.svg" alt="rust-lang necrometer">
"#,
    )
}

pub fn subject_page(r: &Reading) -> String {
    let base = "https://necrometer.dev";
    let path = format!("/{}/{}", r.kind.prefix(), r.subject);
    let snippet = format!("[![Necrometer]({base}{path}.svg)]({base}{path})");
    let mut rows = String::new();
    for c in r
        .entries
        .iter()
        .filter(|c| c.fate != crate::metrics::Fate::Alive)
    {
        rows.push_str(&format!(
            r#"<tr class="fate-{fate}"><td><a href="{url}">{name}</a>{stillborn}</td><td>{d}d</td><td>{s}</td><td>{fl}</td></tr>"#,
            fate = esc_text(c.fate.label()),
            url = esc_text(&c.url),
            name = esc_text(&c.name),
            stillborn = if c.stillborn { r#" <span class="stillborn">stillborn</span>"# } else { "" },
            d = c.days_idle, s = c.stars, fl = esc_text(c.fate.label())
        ));
    }
    let summary = if r.total == 0 {
        String::new()
    } else {
        format!(
            r#"<p class="dim">{t} repos examined · {s} stars stranded{stillborn}</p>"#,
            t = r.total,
            s = r.stars_stranded,
            stillborn = if r.stillborn > 0 {
                format!(" · {} stillborn", r.stillborn)
            } else {
                String::new()
            }
        )
    };
    page(
        &format!("{} — necrometer reading", r.subject),
        &format!(
            r#"
<p><a href="/">← the morgue</a></p>
<h1>@{subj}</h1>
<img src="{path}.svg" alt="necrometer card">
<h2>{idx}% necrotic — {title}</h2>
{summary}
{graveyard}
<h2>haunt your README</h2>
<pre>{snippet}</pre>
<form action="/go" method="get">
<input name="subject" placeholder="check another">
<button type="submit">exhume</button>
</form>
"#,
            subj = esc_text(&r.subject),
            path = path,
            idx = r.index,
            title = esc_text(&r.title),
            summary = summary,
            graveyard = if rows.is_empty() {
                String::new()
            } else {
                format!("<h2>the graveyard</h2><table><tr><th>repo</th><th>idle</th><th>stars</th><th>fate</th></tr>{rows}</table>")
            },
            snippet = esc_text(&snippet)
        ),
    )
}

pub fn not_found(msg: &str) -> String {
    page(
        "not found",
        &format!(
            r#"
<h1>empty plot</h1>
<p>{msg}</p>
<form action="/go" method="get">
<input name="subject" placeholder="try someone else">
<button type="submit">dig</button>
</form>
"#,
            msg = esc_text(msg)
        ),
    )
}

pub fn upstream_error() -> String {
    page(
        "upstream error",
        r#"
<h1>github is not answering</h1>
<p class="dim">try again in a minute.</p>
<p><a href="/">back to the morgue</a></p>
"#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landing_mentions_input() {
        assert!(landing().contains("subject"));
    }

    #[test]
    fn not_found_echoes_msg() {
        let p = not_found("hello");
        assert!(p.contains("hello"));
    }
}
