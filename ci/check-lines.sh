#!/usr/bin/env bash
# Fail if any authored file is over 256 lines.
set -euo pipefail
ROOT=$(git rev-parse --show-toplevel)
MAX=256
over=0
while IFS= read -r -d '' f; do
  rel=${f#"$ROOT"/}
  case "$rel" in
    pkg/*|target/*|.git/*) continue ;;
    Cargo.lock|*/Cargo.lock) continue ;;
    hall.json|*/hall.json) continue ;;
  esac
  n=$(wc -l < "$f")
  if [ "$n" -gt "$MAX" ]; then
    echo "OVER $MAX  $n  $rel"
    over=1
  fi
done < <(find "$ROOT" \( -name .git -o -name target -o -name pkg \) -prune -o \
  -type f \( -name '*.rs' -o -name '*.js' -o -name '*.mjs' -o -name '*.html' \
    -o -name '*.yml' -o -name '*.yaml' -o -name '*.md' -o -name '*.sh' \
    -o -name '*.toml' -o -name 'Dockerfile' -o -name 'action.yml' \) -print0)
if [ "$over" -ne 0 ]; then
  echo "authored files must be <= $MAX lines (see CONTRIBUTING.md)"
  exit 1
fi
echo "ok: no authored file over $MAX lines"
