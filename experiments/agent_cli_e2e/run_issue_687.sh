#!/usr/bin/env bash
# Real Agent CLI reproduction for issue #687. Six separate CLI invocations
# continue one session so the test covers transport, the two report confirmation
# choices, client-side tool execution, history interpretation, and follow-up research.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
BIN_DIR="$(cd "$(dirname "$BIN")" && pwd)"
PORT="${PORT:-8776}"
AGENT="${AGENT:-agent}"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/fake-bin"
GH_LOG="$WORKDIR/gh-invocations.log"

mkdir -p "$FAKE_BIN"
cd "$WORKDIR"

# The context limit below is a HARNESS knob (the server never enforces it) and is
# deliberately far larger than the deterministic fixture transcript. This keeps
# client-side compaction from replacing the pending report prompt, which would
# test Agent's session maintenance instead of Formal AI's continued conversation.
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
          "limit": { "context": 4000000, "output": 65536 }
        }
      }
    }
  },
  "mcp": {
    "issue687": {
      "type": "local",
      "command": ["node", "$ROOT/experiments/agent_cli_e2e/mock-research-mcp.mjs"],
      "enabled": true
    }
  },
  "tools": {
    "websearch": false,
    "webfetch": false
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

# Give this server a PRIVATE, empty memory file. The chat handler feeds the whole
# shared memory log into tool planning and records every exchange back
# (src/server.rs, src/memory_sync.rs), so without isolation the server inherits
# whatever the ~15 earlier E2E scripts in the same CI job recorded into the shared
# ~/.formal-ai/memory.lino. That cross-test state varies run to run and perturbs
# the deterministic planner enough to drop a tool-call round — the exact reason
# the `>= 11 rounds` assertion flaked on main (issue #828). FORMAL_AI_DREAMING=0
# also stops the background compaction thread from mutating this file mid-run.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
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

# Count chat-completion rounds recorded in the server log so far.
posts_so_far() { grep -c 'POST /v1/chat/completions' "$LOG" 2>/dev/null || true; }

# Per-turn round breakdown, so a future boundary failure names the turn that lost
# a round instead of only reporting a total (issue #828, requirement R6). Always
# on because it is cheap; set ROUND_TRACE=0 to silence it.
ROUND_TRACE="${ROUND_TRACE:-1}"
ROUND_BREAKDOWN=""
prev_posts=0

run_turn() {
  local label="$1"
  local prompt="$2"
  shift 2
  echo "== $label: $prompt ==" | tee -a "$AGENT_LOG"
  FORMAL_AI_BASE_URL="http://127.0.0.1:$PORT/v1" \
    FORMAL_AI_DIALOG_LOG_DIR="$WORKDIR/dialog-logs" \
    LINK_ASSISTANT_AGENT_DISABLE_AUTOUPDATE=1 \
    PATH="$FAKE_BIN:$BIN_DIR:$PATH" timeout 180 "$AGENT" \
    --prompt "$prompt" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    "$@" >> "$AGENT_LOG" 2>&1 || fail "$label failed"
  local now_posts turn_posts
  now_posts="$(posts_so_far)"
  turn_posts=$((now_posts - prev_posts))
  prev_posts="$now_posts"
  ROUND_BREAKDOWN+="  $label: $turn_posts round(s) (cumulative $now_posts)"$'\n'
  [ "$ROUND_TRACE" = "0" ] || echo "   -> $label used $turn_posts chat round(s), $now_posts total"
}

run_turn research "When are the next elections in the USA?"
run_turn report "Report this problem" --continue --no-fork
run_turn report_destination "GitHub issue" --continue --no-fork
run_turn report_context "Both logs" --continue --no-fork
run_turn recall "What were we talking about?" --continue --no-fork
run_turn follow_up "Learn about it." --continue --no-fork

if [ ! -f "$GH_LOG" ]; then
  # Distinguish the two ways this can happen: the request never reached the
  # server's report handler because opencode compacted the session (harness
  # problem), versus the handler saw the prompt and declined to act (product
  # problem). Guessing between them is why this took two runs to diagnose.
  if grep -q 'summarizing conversations' "$LOG"; then
    echo "!! diagnosis: the session was auto-summarised mid-run; the report prompt" >&2
    echo "!! was replaced by opencode's own continuation before reaching the server." >&2
  fi
  fail "report request did not execute gh"
fi
grep -q 'issue create --repo link-assistant/formal-ai' "$GH_LOG" \
  || fail "gh invocation did not target the Formal AI repository"
grep -qi 'election' "$AGENT_LOG" \
  || fail "conversation recall did not preserve the election topic"
grep -q '999999' "$AGENT_LOG" \
  || fail "report confirmation did not surface the created issue URL"

# Lower bound is a coarse liveness check, not a determinism assertion: the six
# server-planned turns above already recorded a stable 9-round prefix under
# isolation (research=4, report=1, report_destination=1, report_context=2,
# recall=1); only the trailing `follow_up` turn's round count varies because
# opencode chains a variable number of webfetch calls client-side (issue #828
# analysis, §4). 9 keeps the guard against a genuinely broken workflow (which
# would collapse to ~4-6) while tolerating the observed 11-13 spread and any
# rare further dip from the client's nondeterminism. The strong behavioural
# assertions above (gh targeted formal-ai, election recalled, issue URL
# surfaced, `searches >= 2`) are the real regression guardrails.
posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
if [ "$posts" -lt 9 ]; then
  echo "== per-turn chat-round breakdown ==" >&2
  printf '%s' "$ROUND_BREAKDOWN" >&2
  fail "expected at least 9 chat rounds, got $posts"
fi
searches="$(grep -c 'agentic_outcome: planned ToolCalls.*websearch' "$LOG" || true)"
[ "$searches" -ge 2 ] || fail "initial and contextual research did not both reach websearch"

echo "== issue #687 E2E OK: report executed, recall retained context, follow-up researched it ($posts rounds) =="
echo "== per-turn chat-round breakdown =="
printf '%s' "$ROUND_BREAKDOWN"
tail -80 "$AGENT_LOG"
