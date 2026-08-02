#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI -> generated-artifact proof for one of
# issue #840's five reviewed smallest leaves.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8814}"
AGENT_TIMEOUT_SECONDS="${AGENT_TIMEOUT_SECONDS:-60}"
EVIDENCE_DIR="${EVIDENCE_DIR:-$ROOT/docs/case-studies/issue-840/self-hosting}"
CANONICAL="$ROOT/data/meta/grounded-action-authored-invariant.lino"
WORKDIR="$(mktemp -d)"
SERVER_LOG="$EVIDENCE_DIR/formal-ai.log"
AGENT_LOG="$EVIDENCE_DIR/agent-stream.jsonl"
GENERATED="$EVIDENCE_DIR/grounded-action-authored-invariant.lino"
SERVER_PID=""
TASK='Create file grounded-action-authored-invariant.lino containing
grounded_action_authored_invariant
  record_type meta_invariant
  trigger "empty observation"
  decision "widen within the bounded scope"
  prohibition "do not claim absence"'
EXPECTED_PAYLOAD='grounded_action_authored_invariant
  record_type meta_invariant
  trigger "empty observation"
  decision "widen within the bounded scope"
  prohibition "do not claim absence"'

cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  echo "!! $1" >&2
  tail -100 "$AGENT_LOG" >&2 2>/dev/null
  tail -160 "$SERVER_LOG" >&2 2>/dev/null
  exit 1
}

mkdir -p "$EVIDENCE_DIR"
cat > "$WORKDIR/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
  || fail "Formal AI server never came up"

(
  cd "$WORKDIR"
  timeout "$AGENT_TIMEOUT_SECONDS" "$AGENT" \
    --prompt "$TASK" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    --compaction-model same \
    --output-format stream-json \
    --compact-json \
    --verbose
) > "$AGENT_LOG" 2>&1 || fail "Agent CLI did not complete"

[ -f "$WORKDIR/grounded-action-authored-invariant.lino" ] \
  || fail "Agent CLI did not author the invariant"
[ "$(cat "$WORKDIR/grounded-action-authored-invariant.lino")" = "$EXPECTED_PAYLOAD" ] \
  || fail "Agent CLI artifact differs from the reviewed leaf"
cp "$WORKDIR/grounded-action-authored-invariant.lino" "$GENERATED"

grep -q '"session_id":"ses_' "$AGENT_LOG" \
  || fail "Agent CLI stream did not preserve its session id"
grep -q 'agentic_outcome: planned Final' "$SERVER_LOG" \
  || fail "Formal AI did not finish the self-authoring recipe"
grep -q 'agentic_outcome: planned ToolCalls.*write' "$SERVER_LOG" \
  || fail "Formal AI did not drive a write step"
grep -q 'agentic_outcome: planned ToolCalls.*bash' "$SERVER_LOG" \
  || fail "Formal AI did not verify the authored bytes"

if [ -f "$CANONICAL" ]; then
  [ "$(cat "$GENERATED")" = "$(cat "$CANONICAL")" ] \
    || fail "committed invariant differs from Agent-authored output"
fi

echo "== issue #840 self-authored leaf OK =="
grep -m1 -o '"session_id":"ses_[^"]*"' "$AGENT_LOG"
