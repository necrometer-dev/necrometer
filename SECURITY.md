# Security policy

## Supported versions

Only the latest minor release receives security fixes. Older versions
get best-effort backports if the fix is small.

| Version   | Supported          |
|-----------|--------------------|
| v0.4.x    | :white_check_mark: |
| v0.3.x    | :x:                |
| < v0.3    | :x:                |

## Reporting

Email **security@necrometer.dev** (PGP key on request). Don't open a
public issue for a security bug.

## Threat model

The daemon (`seance serve`):

- Reads public GitHub repo metadata via `api.github.com`.
- Serves SVG / JSON / HTML over HTTP/1.1.
- Has no write access to anything.
- Caches responses in-process for 5 minutes per (subject, kind).

Out of scope: a malicious GitHub response, a malicious upstream CDN,
the host OS. We assume the network is hostile and the user-controlled
inputs (`subject`, `file`) are hostile.

## Mitigations in place

- **TLS pinning.** All outbound HTTPS uses `rustls` with the bundled
  `webpki-roots`. No system trust store, no OpenSSL.
- **Subject validation.** Regex `^[A-Za-z0-9][A-Za-z0-9-]{0,38}$`,
  enforced in the CLI, the Action, and the web server.
- **Output path validation.** The CLI refuses absolute paths, `..`,
  NULs, and shell metacharacters.
- **Rate limiting.** Token-bucket: 60 requests / minute / IP for the
  web server.
- **Security headers.** `Content-Security-Policy`,
  `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: no-referrer`.
- **No upstream error body echo.** The web server returns a generic
  502 on GitHub errors; raw response bodies are logged, not surfaced.
- **Reproducible builds.** `SOURCE_DATE_EPOCH` is honored in the
  Docker build; `cargo-chef` caches only `Cargo.lock`-driven layers.
- **Dependency policy.** `deny.toml` denies unknown registries, unknown
  Git sources, and bans new transitive wildcard versions.

## Acknowledgements

Thanks to every researcher who reports responsibly.