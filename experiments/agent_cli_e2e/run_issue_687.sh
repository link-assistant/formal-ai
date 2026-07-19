#!/usr/bin/env bash
# Real Agent CLI reproduction for issue #687. Four separate CLI invocations
# continue one session so the test covers transport, client-side tool execution,
# server-side history interpretation, and contextual follow-up research.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8776}"
AGENT="${AGENT:-agent}"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/fake-bin"
GH_LOG="$WORKDIR/gh-invocations.log"

mkdir -p "$FAKE_BIN"
cd "$WORKDIR"

cat > opencode.json <<EOF
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
        "formal-ai": {
          "name": "Formal AI Symbolic Production",
          "limit": { "context": 200000, "output": 65536 }
        }
      }
    }
  }
}
EOF

# The report step must execute a shell action, but an E2E test must never create
# a real issue. This PATH-local gh records argv and returns the same issue URL
# shape as GitHub CLI.
cat > "$FAKE_BIN/gh" <<EOF
#!/usr/bin/env bash
printf '%s\n' "\$*" >> "$GH_LOG"
printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/999999'
EOF
chmod +x "$FAKE_BIN/gh"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$WORKDIR"' EXIT

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on port $PORT"
  tail -80 "$LOG"
  exit 1
fi

fail() {
  echo "!! $*" >&2
  echo "== Agent CLI log ==" >&2
  tail -160 "$AGENT_LOG" >&2
  echo "== formal-ai server log ==" >&2
  tail -200 "$LOG" >&2
  exit 1
}

run_turn() {
  local label="$1"
  local prompt="$2"
  shift 2
  echo "== $label: $prompt ==" | tee -a "$AGENT_LOG"
  PATH="$FAKE_BIN:$PATH" timeout 180 "$AGENT" \
    --prompt "$prompt" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    "$@" >> "$AGENT_LOG" 2>&1 || fail "$label failed"
}

run_turn research "When are the next elections in the USA?"
run_turn report "Report this problem" --continue --no-fork
run_turn recall "What were we talking about?" --continue --no-fork
run_turn follow_up "Learn about it." --continue --no-fork

[ -f "$GH_LOG" ] || fail "report request did not execute gh"
grep -q 'issue create --repo link-assistant/formal-ai' "$GH_LOG" \
  || fail "gh invocation did not target the Formal AI repository"
grep -qi 'election' "$AGENT_LOG" \
  || fail "conversation recall did not preserve the election topic"
grep -q '999999' "$AGENT_LOG" \
  || fail "report confirmation did not surface the created issue URL"

posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
[ "$posts" -ge 9 ] || fail "expected at least 9 chat rounds, got $posts"
searches="$(grep -c 'agentic_outcome: planned ToolCalls.*tool: "websearch"' "$LOG" || true)"
[ "$searches" -ge 2 ] || fail "initial and contextual research did not both reach websearch"

echo "== issue #687 E2E OK: report executed, recall retained context, follow-up researched it ($posts rounds) =="
tail -80 "$AGENT_LOG"
