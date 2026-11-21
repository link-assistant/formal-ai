#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
e2e="$root/rust/tests/e2e"
if [[ "${1:-}" == --vsix || "${1:-}" == --npm ]]; then
  archive="${2:?Pass the built package}"
  verifier="${3:?Pass the workflow-owned worker verifier}"
  receipt="${4:?Pass the expected worker receipt digest}"
  extracted="$(mktemp -d)"
  trap 'rm -rf -- "$extracted"' EXIT
  if [[ "$1" == --vsix ]]; then
    unzip -q "$archive" 'extension/dist-web/*' -d "$extracted"
    web="$extracted/extension/dist-web"
    node "$verifier" verify "$PWD" "$web" "$receipt"
  else
    tar -xzf "$archive" -C "$extracted" package/assets
    web="$extracted/package/assets"
    node "$verifier" verify "$PWD" "$web" "$receipt" "$PWD/js/formal_ai_worker.receipt.json"
  fi
  bash "$0" "$web"
  exit 0
fi
if [[ "${1:-}" == --capture ]]; then
  target="${2:?Pass the destination for the workflow-owned smoke test}"
  install -Dm755 "$0" "$target/scripts/verify-browser-artifact.sh"
  for file in .npmrc package.json package-lock.json playwright.pages{,.smoke}.config.js \
    tests/pages-live-smoke.spec.js tests/support/worker-{ready,readiness-budget,diagnostics}.js; do
    install -Dm644 "$e2e/$file" "$target/rust/tests/e2e/$file"
  done
  for file in package.json index.cjs index.mjs; do
    install -Dm644 "$root/packages/minimatch-compat/$file" "$target/packages/minimatch-compat/$file"
  done
  exit 0
fi
artifact="$(realpath "${1:?Pass the built web resources}")"
port="${2:-3456}"
npm ci --prefix "$e2e" --no-audit --no-fund
cd "$e2e"
npx playwright install chromium
log="$(mktemp)"
python3 -m http.server "$port" --bind 127.0.0.1 --directory "$artifact" > "$log" 2>&1 &
server=$!
cleanup() {
  kill "$server" 2>/dev/null || true
  wait "$server" 2>/dev/null || true
  rm -f -- "$log"
}
trap cleanup EXIT
if ! curl -fsS --retry 5 --retry-connrefused "http://127.0.0.1:$port/app/" >/dev/null; then
  cat "$log" >&2
  exit 1
fi
PAGES_URL="http://127.0.0.1:$port/" npx playwright test --config=playwright.pages.smoke.config.js
