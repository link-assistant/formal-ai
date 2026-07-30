#!/usr/bin/env bash
# Reproduce the deterministic two-client comparison ledger against a running server.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
SANDBOX="$(mktemp -d)"
WORKSPACE="$SANDBOX/workspace"
FAKE_BIN="$SANDBOX/bin"
trap 'rm -rf "$SANDBOX"' EXIT

mkdir -p "$FAKE_BIN" "$WORKSPACE"
ln -s "$ROOT/experiments/issue_703_orchestration/scripted-client.sh" "$FAKE_BIN/codex"
ln -s "$ROOT/experiments/issue_703_orchestration/scripted-client.sh" "$FAKE_BIN/claude"
printf '# Before\n' > "$WORKSPACE/README.md"

PATH="$FAKE_BIN:$PATH" "$BIN" agent dispatch \
  --cli codex,claude \
  --compare \
  --task "add a README badge" \
  --workspace "$WORKSPACE" \
  --base-url "$BASE_URL" \
  --allow-command test \
  --verify '["test","-s","README.md"]'

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR/sessions"
  cp "$WORKSPACE/.formal-ai-orchestration/comparison-ledger.json" "$ARTIFACT_DIR/"
  cp "$WORKSPACE/.formal-ai-orchestration/sessions/"*.json "$ARTIFACT_DIR/sessions/"
fi
