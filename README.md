<div align="center">

# ☠ Necrometer

**measures how dead your GitHub is.**

[![Necrometer](necrometer.svg)](https://necrometer.dev/?u=necrometer-dev)

### → [necrometer.dev](https://necrometer.dev) ←

*one crate. two bodies. zero servers.*

</div>

---

This is the engine — a single Rust crate that is the source of truth for all
necromancy. It compiles to three forms:

| form | what it does |
|---|---|
| **native CLI** | what the GitHub Actions run — `card` and `hall` subcommands |
| **wasm** | what [necrometer.dev](https://necrometer.dev) runs in your browser — `analyze_repos` + `render_card` |
| **axum service** | the dormant self-hosted variant (live `/u/{name}.svg` endpoints), if you ever want it |

## how it reads the dead

Every non-fork repo gets a **fate** from its last push:

| fate | last activity |
|---|---|
| alive | < 30 days |
| cooling | < 180 days |
| cold | < 2 years |
| dead | older, or stillborn (created and never pushed) |
| buried | archived |

The **necrosis index** is the weighted average, 0–100. Titles range from
*The Maintainer* to *Repo Necromancer* ("not a profile — a cemetery").

## the card

The generated `necrometer.svg` is self-contained: SMIL-animated needle, EKG
that flatlines into a skull, mood-based skull buddy, a tiny graveyard of your
corpses — flowers if nothing has died. It renders inside README `<img>` embeds
because GitHub allows declarative SVG animation (scripts are blocked; SMIL is not).

## install (the Action does this for you)

```sh
REL=https://github.com/necrometer-dev/necrometer/releases/download/v0.2.0
curl -sSLO $REL/necrometer-x86_64-unknown-linux-musl.tar.gz
curl -sSLO $REL/SHA256SUMS
sha256sum -c SHA256SUMS
tar xzf necrometer-x86_64-unknown-linux-musl.tar.gz
```

## usage

```sh
# carve a card
./necrometer card torvalds necrometer.svg

# mass grave census — one name per line
./necrometer hall names.txt hall.json

# summon the dormant service
./necrometer serve   # :8080 — /u/{name}, /u/{name}.svg
```

Tokens: `NECRO_TOKEN`, `GH_TOKEN`, or `GITHUB_TOKEN` env vars are picked up
automatically — required to see private repos, and to see orgs at all when
you're not public about membership.

## put it on your own README

The card is a committed file, not a hosted image — necrometer hosts nothing
per-user. The auto-updating ward is a workflow that runs the release binary
on GitHub's compute and re-commits the card daily. The site generates the
exact workflow + markdown for you:

**→ [necrometer.dev](https://necrometer.dev) — type a name, copy the ritual.**

Or steal `.github/workflows/necrometer.yml` from this repo — it's the same one.

## wasm

`pkg/` is built with [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```sh
wasm-pack build --target web --out-dir pkg
```

```js
import init, { analyze_repos, render_card } from './pkg/necrometer.js';
await init();
const reading = analyze_repos('defunkt', JSON.stringify(repos)); // repos from api.github.com
const svg = render_card(reading);
```

## layout

```
src/metrics.rs   fates, necrosis index, titles, flavor — the judgment
src/card.rs      the SVG — gauge, skull buddy, graveyard, EKG
src/github.rs    Repo struct + native API client (org/self/public resolution)
src/wasm.rs      wasm-bindgen exports
src/web.rs       the dormant axum service
src/main.rs      CLI: card / hall / serve
```

---

<div align="center">
<sub>no servers were harmed. they were already dead.</sub>
</div>
