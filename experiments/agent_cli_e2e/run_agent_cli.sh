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
#   EXPECT_SERVER_TEXTS
#                 Optional newline-separated strings that must appear in the
#                 server trace (useful for proving exact harness-side commands).
#   ARTIFACT_DIR  Optional directory receiving the server log, Agent CLI log,
#                 and generated file after a successful live replay.
#   ATTEMPTS      How many times to (re)drive the CLI before giving up (default: 5).
#                 The third-party CLI is non-deterministic — see the retry note
#                 below — so a stalled first attempt is retried, not fatal.
#
# The script exits non-zero (with a diagnostic tail of the server log and the
# CLI stdout/stderr) if:
#   - the server never comes up on PORT
#   - the CLI exits non-zero on the final attempt
#   - EXPECT_FILE is still missing from the workdir after ATTEMPTS runs
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
EXPECT_SERVER_TEXTS="${EXPECT_SERVER_TEXTS:-}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
# Minimum /v1/chat/completions round-trips the recipe must drive. The default (4)
# fits the web recipes (search → fetch → write → verify → final = 5 posts). A
# no-web recipe (e.g. the diagram task: write → verify → final = 3 posts) sets
# MIN_POSTS=3, so the same harness validates every recipe axis live rather than
# only the web ones.
MIN_POSTS="${MIN_POSTS:-4}"

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
        "formal-ai": { "name": "Formal AI Symbolic Production" }
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
#
# The external `@link-assistant/agent` CLI is *non-deterministic*: it
# occasionally exits 0 after only the first tool round (a websearch) without
# walking the rest of the recipe, so no file is written. That is a property of
# the third-party CLI — the deterministic formal-ai server plans the same next
# step every time (visible in the server trace) — so we retry the whole
# invocation up to ATTEMPTS times and stop as soon as EXPECT_FILE appears. A
# stalled attempt exits in a few seconds, so the retries stay well inside the
# job timeout, and every hard assertion below still has to pass on a genuine,
# complete round-trip that actually wrote the file.
ATTEMPTS="${ATTEMPTS:-5}"
RC=1
for attempt in $(seq 1 "$ATTEMPTS"); do
  echo "== agent attempt $attempt/$ATTEMPTS =="
  timeout 180 "$AGENT" run \
    --prompt "$TASK" \
    --disable-stdin \
    --model "formal-ai/formal-ai" \
    > "$AGENT_LOG" 2>&1
  RC=$?
  echo "== agent exit: $RC =="
  if [ "$RC" -eq 0 ] && [ -f "$WORKDIR/$EXPECT_FILE" ]; then
    break
  fi
  echo "== attempt $attempt produced no $EXPECT_FILE (external CLI stalled?); retrying =="
done

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

# One extra structural check: the server must have seen at least MIN_POSTS
# /v1/chat/completions posts — a single post would mean the loop stopped after
# the first turn without walking the recipe (search → fetch → write → verify).
# The count is cumulative across retries, but since the successful attempt that
# wrote EXPECT_FILE necessarily walked the full recipe, it alone contributes
# ≥MIN_POSTS, so this stays a valid lower bound.
posts="$(grep -c 'POST /v1/chat/completions' "$LOG" || true)"
[ "$posts" -ge "$MIN_POSTS" ] \
  || fail "expected ≥$MIN_POSTS chat completions, got $posts (loop stalled?)"

while IFS= read -r expected; do
  [ -z "$expected" ] && continue
  grep -Fq "$expected" "$LOG" \
    || fail "expected server trace to contain: $expected"
done <<< "$EXPECT_SERVER_TEXTS"

echo "== E2E OK: $EXPECT_FILE written, contains \"$EXPECT_TEXT\", $posts chat rounds =="
head -5 "$WORKDIR/$EXPECT_FILE"
if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$AGENT_LOG" "$ARTIFACT_DIR/agent-cli.log"
  cp "$WORKDIR/$EXPECT_FILE" "$ARTIFACT_DIR/$EXPECT_FILE"
fi
