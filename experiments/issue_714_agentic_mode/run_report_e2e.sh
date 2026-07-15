#!/usr/bin/env bash
# Drive the real Agent CLI through Formal AI and prove that Report executes `gh`.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8780}"
WORKDIR="$(mktemp -d)"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
CAPTURE="$WORKDIR/gh-invocation.txt"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

FAKE_BIN="$ROOT/experiments/issue_714_agentic_mode/fixtures"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SERVER_PID=$!
curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null

cd "$WORKDIR"
CONFIG="{\"provider\":{\"formal-ai\":{\"npm\":\"@ai-sdk/openai-compatible\",\"name\":\"Formal AI\",\"options\":{\"baseURL\":\"http://127.0.0.1:$PORT/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formal-ai/formal-ai\"}"
FORMAL_AI_GH_CAPTURE="$CAPTURE" PATH="$FAKE_BIN:$PATH" \
  FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$CONFIG" \
  "$AGENT" run \
    --prompt "Report issue" \
    --disable-stdin \
    --model "formal-ai/formal-ai" \
    > "$AGENT_LOG" 2>&1

test -f "$CAPTURE"
grep -Fxq issue "$CAPTURE"
grep -Fxq create "$CAPTURE"
grep -Fxq link-assistant/formal-ai "$CAPTURE"
grep -q 'issues/999' "$AGENT_LOG"
posts="$(grep -c 'POST /v1/chat/completions' "$LOG")"
test "$posts" -ge 2

echo "Agent CLI invoked gh successfully in $posts chat rounds."
