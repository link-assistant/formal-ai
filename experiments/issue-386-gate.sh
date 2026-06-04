#!/bin/sh
# Issue #386 full CI gate. Run from repo root. Writes per-step logs to /tmp.
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "== fmt =="
cargo fmt --all -- --check && echo FMT_CLEAN
echo "== clippy =="
cargo clippy --all-targets --all-features 2>&1 | tail -n 20
echo "== lib tests =="
cargo test -p formal-ai --lib 2>&1 | tail -n 15
echo "== unit tests =="
cargo test -p formal-ai --test unit 2>&1 | tail -n 15
echo "== worker sync (regenerate MEANINGS_LINO) =="
node experiments/issue-386-sync-worker-lexicon.mjs 2>&1 | tail -n 5
echo "== mirror verify =="
node experiments/issue-386-meanings-mirror.mjs 2>&1 | tail -n 5
echo "== e2e checks =="
for c in check:i18n check:language-parity check:language-test-coverage check:intent-coverage check:web-tdz; do
  echo "-- $c --"
  npm run --prefix tests/e2e "$c" 2>&1 | tail -n 4
done
echo "ALL_GATE_DONE"
