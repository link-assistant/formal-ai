#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
bun_bin=${BUN_BIN:-bun}
iterations=${1:-10}
bundles=(
  src/web/vendor.bundle.js
  src/web/web-search-component.bundle.js
  src/web/ocr.bundle.js
  src/web/app.js
)

cd "$repo_dir"
if [[ "$bun_bin" == */* ]]; then
  bun_dir=$(cd "$(dirname "$bun_bin")" && pwd)
  export PATH="$bun_dir:$PATH"
  bun_bin="$bun_dir/$(basename "$bun_bin")"
fi

"$bun_bin" run build:web >/dev/null
expected=$(
  sha256sum "${bundles[@]}"
)

for ((iteration = 1; iteration <= iterations; iteration++)); do
  "$bun_bin" run build:web >/dev/null
  actual=$(
    sha256sum "${bundles[@]}"
  )
  if [[ "$actual" != "$expected" ]]; then
    diff -u <(printf '%s\n' "$expected") <(printf '%s\n' "$actual")
    exit 1
  fi
  printf 'iteration %d: stable\n' "$iteration"
done
