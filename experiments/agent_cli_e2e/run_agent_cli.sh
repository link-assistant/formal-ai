#!/usr/bin/env bash
# Real Agent-CLI ↔ formal-ai E2E round-trip: boot `formal-ai serve`, drive it
# with the real `@link-assistant/agent` CLI (talking over its OpenAI-compatible
# HTTP/SSE endpoint) and prove the CLI actually writes the requested file.
#
# Usage:
#   experiments/agent_cli_e2e/run_agent_cli.sh
#
# Environment knobs:
#   BIN           Path to the release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT          Server port (default: 8763)
#   AGENT         Path to the agent CLI (default: `agent` on PATH)
#   TASK          The user prompt for the CLI (default: the canonical #538 seed)
#   EXPECT_FILE   File the CLI is expected to write inside the sandbox workdir
#                 (default: meanings-tomato-detail.lino)
#   EXPECT_TEXT   A string that must appear inside EXPECT_FILE (default: `томаты`,
#                 the previously missing Russian plural — the issue's canary)
#
# The script exits non-zero (with a diagnostic tail of the server log and the
# CLI stdout/stderr) if:
#   - the server never comes up on PORT
#   - the CLI exits non-zero
#   - EXPECT_FILE is missing from the workdir at the end of the run
#   - EXPECT_TEXT is missing from EXPECT_FILE
#
# This is the exact loop CI runs (see .github/workflows/release.yml).

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8763}"
AGENT="${AGENT:-agent}"
# Default TASK is assigned in two steps because bash `${VAR:-default}` still
# tracks quote-pairing inside the default word — an unescaped apostrophe in the
# default (`surface's`) triggers "unexpected EOF" at parse time. Assigning the
# default to a plain double-quoted variable first sidesteps that quirk while
# keeping the outer `TASK="${TASK:-$DEFAULT_TASK}"` env-override behaviour.
DEFAULT_TASK="Make the tomato meaning more detailed: pin every surface's part of speech and grammatical number, ground it in Wikidata, and add the missing plural to томат."
TASK="${TASK:-$DEFAULT_TASK}"
EXPECT_FILE="${EXPECT_FILE:-meanings-tomato-detail.lino}"
EXPECT_TEXT="${EXPECT_TEXT:-томаты}"

LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"

echo "== workdir: $WORKDIR =="
cd "$WORKDIR"

# opencode.json wires the CLI to our OpenAI-compatible server under a custom
# provider id (`formal-ai`). `npm: "@ai-sdk/openai-compatible"` picks Vercel's
# generic OpenAI-compatible adapter — the CLI POSTs to /v1/chat/completions and
# reads back the streamed chat.completion.chunk SSE the server now emits.
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
        "formal-symbolic-production": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF

# FORMAL_AI_AGENT_MODE=1 flips the permission gate on for tool-call execution
# (see AssociativePackage / permission_for_capability). FORMAL_AI_TRACE_REQUESTS=1
# adds a request trace to the server log so a failed run has visible planner
# state to diagnose.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$WORKDIR"' EXIT

# Wait for /health without a foreground sleep (curl retries handle the backoff).
if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on port $PORT"
  echo "== server log =="
  tail -60 "$LOG"
  exit 1
fi
echo "== server up on $PORT =="

# `--disable-stdin` prevents the CLI from opening its interactive prompt (this
# script drives a single prompt through). 180s is generous for a 4-step loop
# where each POST is deterministic and finishes in <100ms — the extra time
# absorbs npm-install setup on a cold CI runner.
timeout 180 "$AGENT" run \
  --prompt "$TASK" \
  --disable-stdin \
  --model "formal-ai/formal-symbolic-production" \
  > "$AGENT_LOG" 2>&1
RC=$?
echo "== agent exit: $RC =="

echo "== agent stderr/out tail =="
tail -40 "$AGENT_LOG"

echo "== server log tail =="
tail -100 "$LOG"

echo "== files in workdir =="
ls -la "$WORKDIR"

fail() {
  echo "!! $*" >&2
  exit 1
}

# The four hard assertions the round-trip has to satisfy.
[ "$RC" -eq 0 ] || fail "agent CLI exited $RC (see $AGENT_LOG)"
[ -f "$WORKDIR/$EXPECT_FILE" ] || fail "expected file $EXPECT_FILE not in workdir"
grep -q "$EXPECT_TEXT" "$WORKDIR/$EXPECT_FILE" \
  || fail "expected text \"$EXPECT_TEXT\" missing from $EXPECT_FILE"

# One extra structural check: the server must have seen more than one
# /v1/chat/completions post — a single post would mean the loop stopped after
# the first turn without walking the recipe (search → fetch → write → verify).
posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
[ "$posts" -ge 4 ] || fail "expected ≥4 chat completions, got $posts (loop stalled?)"

echo "== E2E OK: $EXPECT_FILE written, contains \"$EXPECT_TEXT\", $posts chat rounds =="
head -5 "$WORKDIR/$EXPECT_FILE"
