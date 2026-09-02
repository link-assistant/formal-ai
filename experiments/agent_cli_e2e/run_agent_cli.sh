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
#   EXPECT_FILES  Optional newline-separated additional files that must exist.
#   EXPECT_TEXT   A string that must appear inside EXPECT_FILE (default: `томаты`,
#                 the previously missing Russian plural — the issue's canary)
#   FOLLOW_UP     Optional second prompt. When set, the harness resumes the
#                 first prompt's session without forking before asserting files.
#   EXPECT_SERVER_TEXTS
#                 Optional newline-separated strings that must appear in the
#                 server trace (useful for proving exact harness-side commands).
#   EXPECT_AGENT_TEXTS
#                 Optional newline-separated strings that must appear in the
#                 Agent trace (useful for proving exact command output).
#   ARTIFACT_DIR  Optional directory receiving the server log, Agent CLI log,
#                 and generated file after a successful live replay.
#   RESEARCH_MCP_FIXTURE
#                 Optional repository-relative path to a deterministic MCP
#                 server that replaces Agent's hosted websearch/webfetch tools.
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
EXPECT_FILES="${EXPECT_FILES:-}"
EXPECT_TEXT="${EXPECT_TEXT:-томаты}"
FOLLOW_UP="${FOLLOW_UP:-}"
EXPECT_SERVER_TEXTS="${EXPECT_SERVER_TEXTS:-}"
EXPECT_AGENT_TEXTS="${EXPECT_AGENT_TEXTS:-}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
RESEARCH_MCP_FIXTURE="${RESEARCH_MCP_FIXTURE:-}"
research_mcp_path=""
if [ -n "$RESEARCH_MCP_FIXTURE" ]; then
  case "$RESEARCH_MCP_FIXTURE" in
    /*) research_mcp_path="$RESEARCH_MCP_FIXTURE" ;;
    *) research_mcp_path="$ROOT/$RESEARCH_MCP_FIXTURE" ;;
  esac
  if [ ! -f "$research_mcp_path" ]; then
    echo "!! research MCP fixture not found: $research_mcp_path" >&2
    exit 1
  fi
fi
# Minimum /v1/chat/completions round-trips the recipe must drive. The default (4)
# fits the web recipes (search → fetch → write → verify → final = 5 posts). A
# no-web recipe (e.g. the diagram task: write → verify → final = 3 posts) sets
# MIN_POSTS=3, so the same harness validates every recipe axis live rather than
# only the web ones.
MIN_POSTS="${MIN_POSTS:-4}"

LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
AGENT_SETUP_LOG="/tmp/agent-setup-$PORT.log"
AGENT_FOLLOW_UP_LOG="/tmp/agent-follow-up-$PORT.log"
WORKDIR="$(mktemp -d)"
SERVER_STATE="$(mktemp -d)"

echo "== workdir: $WORKDIR =="
cd "$WORKDIR" || exit 1

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

# Meaning-detail E2Es must exercise search and fetch, but Agent's built-in
# `websearch` delegates to a hosted provider outside this repository. When a
# fixture is requested, keep the full tool loop while replacing both built-ins
# with repository-owned MCP tools. The normal harness path is unchanged.
if [ -n "$RESEARCH_MCP_FIXTURE" ]; then
  node - "$research_mcp_path" <<'NODE'
const fs = require("node:fs");

const configPath = "opencode.json";
const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
config.mcp = {
  agent_cli_e2e: {
    type: "local",
    command: ["node", process.argv[2]],
    enabled: true,
  },
};
config.tools = { websearch: false, webfetch: false };
fs.writeFileSync(configPath, `${JSON.stringify(config, null, 2)}\n`);
NODE
fi

# FORMAL_AI_AGENT_MODE=1 flips the permission gate on for tool-call execution
# (see AssociativePackage / permission_for_capability). FORMAL_AI_TRACE_REQUESTS=1
# adds a request trace to the server log so a failed run has visible planner
# state to diagnose.
# Private, empty memory per run so the chat handler's memory-fed planning and the
# `POST /v1/chat/completions` round count stay deterministic and independent of
# what earlier E2E scripts recorded into the shared ~/.formal-ai/memory.lino
# (issue #828). Keep server-owned state outside WORKDIR: link-cli writes a binary
# sidecar beside the LiNo file, and Agent must not mistake either file for an
# authored workspace effect. FORMAL_AI_DREAMING=0 stops background compaction.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$SERVER_STATE/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$WORKDIR" "$SERVER_STATE"' EXIT

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
# script drives a single prompt through). `--no-summarize-session` and
# `--compaction-model same` keep the round-trip on the formal-ai provider under
# test: Agent otherwise may call an unrelated hosted model between local tool
# turns. 180s is generous for a 4-step loop where each POST is deterministic
# and finishes in <100ms — the extra time absorbs npm-install setup on a cold CI
# runner.
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
  rm -f "$AGENT_LOG" "$AGENT_SETUP_LOG" "$AGENT_FOLLOW_UP_LOG"
  timeout 180 "$AGENT" run \
    --prompt "$TASK" \
    --disable-stdin \
    --no-summarize-session \
    --compaction-model same \
    --model "formal-ai/formal-ai" \
    > "$AGENT_SETUP_LOG" 2>&1
  RC=$?
  cp "$AGENT_SETUP_LOG" "$AGENT_LOG"
  if [ "$RC" -eq 0 ] && [ -n "$FOLLOW_UP" ]; then
    session="$(grep -m1 -o 'ses_[A-Za-z0-9]*' "$AGENT_SETUP_LOG" || true)"
    if [ -z "$session" ]; then
      RC=1
      echo "!! first turn did not report a resumable session id" >> "$AGENT_LOG"
    else
      echo "== resuming session $session =="
      timeout 180 "$AGENT" run \
        --resume "$session" \
        --no-fork \
        --prompt "$FOLLOW_UP" \
        --disable-stdin \
        --no-summarize-session \
        --compaction-model same \
        --model "formal-ai/formal-ai" \
        > "$AGENT_FOLLOW_UP_LOG" 2>&1
      RC=$?
      {
        echo
        echo "== resumed follow-up: $session =="
        cat "$AGENT_FOLLOW_UP_LOG"
      } >> "$AGENT_LOG"
    fi
  fi
  echo "== agent exit: $RC =="
  missing_file=""
  if [ ! -f "$WORKDIR/$EXPECT_FILE" ]; then
    missing_file="$EXPECT_FILE"
  fi
  while IFS= read -r expected_file; do
    [ -z "$expected_file" ] && continue
    if [ ! -f "$WORKDIR/$expected_file" ]; then
      missing_file="$expected_file"
      break
    fi
  done <<< "$EXPECT_FILES"
  if [ "$RC" -eq 0 ] && [ -z "$missing_file" ]; then
    break
  fi
  echo "== attempt $attempt did not complete $missing_file (external CLI stalled?); retrying =="
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
while IFS= read -r expected_file; do
  [ -z "$expected_file" ] && continue
  [ -f "$WORKDIR/$expected_file" ] \
    || fail "expected file $expected_file not in workdir"
done <<< "$EXPECT_FILES"
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

while IFS= read -r expected; do
  [ -z "$expected" ] && continue
  grep -Fq "$expected" "$AGENT_LOG" \
    || fail "expected Agent trace to contain: $expected"
done <<< "$EXPECT_AGENT_TEXTS"

echo "== E2E OK: $EXPECT_FILE written, contains \"$EXPECT_TEXT\", $posts chat rounds =="
head -5 "$WORKDIR/$EXPECT_FILE"
if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$AGENT_LOG" "$ARTIFACT_DIR/agent-cli.log"
  cp "$WORKDIR/$EXPECT_FILE" "$ARTIFACT_DIR/$EXPECT_FILE"
  while IFS= read -r expected_file; do
    [ -z "$expected_file" ] && continue
    mkdir -p "$ARTIFACT_DIR/$(dirname "$expected_file")"
    cp "$WORKDIR/$expected_file" "$ARTIFACT_DIR/$expected_file"
  done <<< "$EXPECT_FILES"
fi
