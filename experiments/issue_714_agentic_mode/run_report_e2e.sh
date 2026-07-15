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
MEMORY="$WORKDIR/memory.lino"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

FAKE_BIN="$ROOT/experiments/issue_714_agentic_mode/fixtures"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 FORMAL_AI_MEMORY_PATH="$MEMORY" \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SERVER_PID=$!
curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null

cd "$WORKDIR"
CONFIG="{\"provider\":{\"formal-ai\":{\"npm\":\"@ai-sdk/openai-compatible\",\"name\":\"Formal AI\",\"options\":{\"baseURL\":\"http://127.0.0.1:$PORT/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formal-ai/formal-ai\"}"
printf '%s\n' "$CONFIG" > opencode.json

RC=1
for attempt in 1 2; do
  set +e
  FORMAL_AI_GH_CAPTURE="$CAPTURE" PATH="$FAKE_BIN:$PATH" \
    LINK_ASSISTANT_AGENT_DISABLE_AUTOUPDATE=1 \
    timeout 60 "$AGENT" run \
      --prompt "Report issue" \
      --disable-stdin \
      --model "formal-ai/formal-ai" \
      > "$AGENT_LOG" 2>&1
  RC=$?
  set -e
  if [[ "$RC" -eq 0 && -f "$CAPTURE" ]]; then
    break
  fi
  echo "Agent CLI report attempt $attempt failed with exit $RC; retrying."
done

echo "== agent stderr/out tail =="
tail -40 "$AGENT_LOG"
echo "== relevant server trace =="
awk '/formal-ai server listening|\[trace\] (GET|POST)|agentic_outcome/' "$LOG" | tail -40

test "$RC" -eq 0
test -f "$CAPTURE"
grep -Fxq issue "$CAPTURE"
grep -Fxq create "$CAPTURE"
grep -Fxq link-assistant/formal-ai "$CAPTURE"
grep -q 'issues/999' "$AGENT_LOG"
posts="$(grep -c 'POST /v1/chat/completions' "$LOG")"
test "$posts" -ge 2
test -f "$MEMORY"
grep -Fq 'kind "tool_call"' "$MEMORY"
grep -Fq 'tool "bash"' "$MEMORY"
grep -Fq 'gh issue create' "$MEMORY"
grep -Fq 'issues/999' "$MEMORY"

echo "Agent CLI invoked gh and retained its result as learning evidence in $posts chat rounds."
