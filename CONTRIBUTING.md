# Contributing to Seance

Thanks for your interest. The project is small and the constraints are
deliberate; please read this before opening a PR.

## Principles (in order)

1. **Zero third-party runtime deps.** The native build links only
   `rustls` + `webpki-roots`. The wasm build links only `wasm-bindgen`.
   Anything else must be hand-rolled under `src/`. Adding a runtime dep
   is a breaking change and needs a one-line justification.
2. **256-line-per-file ceiling.** Every file under `src/` must be ≤256
   lines (including tests). Split by concept; don't grow monoliths.
3. **Zero-trust / first principles.** Don't trust the network, the
   filesystem, the env, the input. Validate at every boundary.
4. **No clever macros, no global statics.** `format!` is fine; proc
   macros aren't worth the compile time.

## Repo layout

```
src/
  card/        # SVG renderer — one concept per file
  metrics/     # necro analysis (analyze, titles, summary)
  github/      # repo model + native HTTP client
  web/         # HTTP/1.1 server (gated on `--features serve`)
  json/        # recursive-descent JSON parser + Value
  http.rs      # rustls blocking client
  http_server.rs
  time.rs      # SystemTime + RFC3339 (native) / Date.now() (wasm)
  escape.rs    # esc_text / is_valid_subject / is_safe_out_path
  error.rs     # Error enum
  wasm.rs      # browser bridge (cfg-gated)
container/     # Alpine + musl Docker build
.github/       # workflows + CODEOWNERS
deny.toml      # cargo-deny policy
```

## Building

```sh
# native CLI
cargo build --release --bin seance
./target/release/seance card torvalds torvalds.svg

# web server (default Docker target)
cargo build --release --bin seance --features serve
./target/release/seance serve --bind 0.0.0.0:8080

# wasm (output goes to pkg/)
wasm-pack build --release --target web --out-dir pkg

# musl static binary (used by the Action)
cargo build --release --target x86_64-unknown-linux-musl --bin seance
```

## Testing

```sh
cargo test --lib                  # 47 unit tests across the modules
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo audit                       # known CVEs in the dep graph
cargo deny check                  # licenses, sources, multiple versions
```

The CI workflow at `.github/workflows/ci.yml` runs all of the above
plus a wasm smoke test. Local reproduction:

```sh
wasm-pack build --release --target web --out-dir pkg
node ci/wasm-smoke.mjs
```

## Bumping the version

`Cargo.toml` is the single source of truth. Bump it, then tag:

```sh
git tag -a vX.Y.Z -m "vX.Y.Z: ..."
git push origin vX.Y.Z
```

The release workflow builds the musl tarball + SHA256SUMS, and the
Action at `necrometer-dev/necrometer-action` pins to that tag.

## Security

See `SECURITY.md`. For private disclosure, see the email there — please
don't open a public issue for a security bug.