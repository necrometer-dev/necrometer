#!/usr/bin/env bash
# Checksum-pinned wasm-pack. Do not curl the installer init.sh.
set -euo pipefail
VER=0.13.1
SHA=c539d91ccab2591a7e975bcf82c82e1911b03335c80aa83d67ad25ed2ad06539
URL="https://github.com/rustwasm/wasm-pack/releases/download/v${VER}/wasm-pack-v${VER}-x86_64-unknown-linux-musl.tar.gz"
DEST="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$DEST" /tmp/wasm-pack-pin
curl --fail --proto '=https' --tlsv1.2 --silent --show-error --location \
  --output /tmp/wasm-pack-pin/wasm-pack.tgz "$URL"
echo "${SHA}  /tmp/wasm-pack-pin/wasm-pack.tgz" | sha256sum --strict --check
tar --extract --gzip --file /tmp/wasm-pack-pin/wasm-pack.tgz -C /tmp/wasm-pack-pin
install -m 0755 \
  "/tmp/wasm-pack-pin/wasm-pack-v${VER}-x86_64-unknown-linux-musl/wasm-pack" \
  "$DEST/wasm-pack"
if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$DEST" >> "$GITHUB_PATH"
fi
echo "wasm-pack ${VER} -> ${DEST}/wasm-pack"
