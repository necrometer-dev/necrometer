//! Inline CSS for the HTML pages. Kept in its own file so it doesn't
//! bloat pages.rs.

pub const CSS: &str = r#"
body { background:#0d1117; color:#e6edf3; font-family:-apple-system,'Segoe UI',Roboto,Helvetica,Arial,sans-serif; margin:0 }
main { max-width:640px; margin:48px auto; padding:0 16px }
h1 { letter-spacing:3px; font-size:26px }
h2 { font-size:16px; color:#8b949e; letter-spacing:1px; text-transform:uppercase; margin-top:32px }
a { color:#a371f7 }
.tag { color:#8b949e }
.dim { color:#8b949e }
img { display:block; margin:12px 0; max-width:100% }
form { margin:24px 0 }
input { background:#161b22; border:1px solid #30363d; color:#e6edf3; padding:10px 12px; border-radius:6px; width:240px; font-size:14px }
button { background:#a371f7; color:#0d1117; border:0; padding:10px 16px; border-radius:6px; font-weight:600; cursor:pointer }
pre { background:#161b22; border:1px solid #30363d; padding:12px; border-radius:6px; overflow-x:auto; font-size:12px }
code { background:#161b22; padding:2px 5px; border-radius:4px; font-size:12px }
.hint { color:#8b949e; font-size:13px }
table { width:100%; border-collapse:collapse; font-size:14px }
th { text-align:left; color:#8b949e; font-weight:500; padding:6px 8px; border-bottom:1px solid #30363d }
td { padding:6px 8px; border-bottom:1px solid #21262d }
.fate-dead td:last-child, .fate-buried td:last-child { color:#f85149 }
.fate-cold td:last-child { color:#d29922 }
.fate-cooling td:last-child { color:#58a6ff }
.stillborn { color:#8b949e; font-size:11px; font-style:italic }
"#;