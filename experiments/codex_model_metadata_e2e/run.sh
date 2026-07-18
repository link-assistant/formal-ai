#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
PORT="${PORT:-8750}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
RUN_DIR="$(mktemp -d)"
SERVER_LOG="$RUN_DIR/formal-ai.log"
CODEX_LOG="$RUN_DIR/codex.log"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true; rm -rf "$RUN_DIR"' EXIT

curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null

"$BIN" with --no-start-server --base-url "http://127.0.0.1:$PORT" \
  codex "hi" >"$CODEX_LOG" 2>&1

if grep -Fq "Model metadata for formal-ai not found" "$CODEX_LOG"; then
  echo "Codex rejected the Formal AI model metadata" >&2
  exit 1
fi
grep -Fq "model: formal-ai" "$CODEX_LOG"
grep -Fq "slug=formal-ai" "$CODEX_LOG"
grep -Fq "POST /api/openai/v1/responses" "$SERVER_LOG"

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$CODEX_LOG" "$ARTIFACT_DIR/codex.log"
fi

echo "Codex accepted Formal AI model metadata and completed a Responses API round trip."
