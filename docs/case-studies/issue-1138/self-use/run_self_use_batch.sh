#!/usr/bin/env bash
# Wave F self-use harness (plan 14 wave F).
#
# Drives the *real* `@link-assistant/agent` CLI against a locally served
# Formal AI binary, one prompt at a time, and records every raw transcript.
# Nothing here repairs a failure: the harness only observes and captures.
#
# Usage:
#   run_self_use_batch.sh <cases.tsv> <out-root>
#
# cases.tsv columns (tab separated, no header):
#   slug <TAB> lang <TAB> prompt
#
# Environment:
#   BIN     release binary
#   PORT    server port (default: 8911)
#   AGENT   agent CLI (default: `agent` on PATH)
#   TIMEOUT per-prompt timeout in seconds (default: 120)
#   SEED_WORKSPACE  optional directory copied into each throwaway workspace
#                   before the prompt runs (used by the repository / toolchain
#                   families that need files to act on)

set -uo pipefail

ROOT="/Users/konard/Code/Archive/link-assistant/formal-ai/.claude/worktrees/issue-1138"
BIN="${BIN:-/Users/konard/Code/Archive/link-assistant/formal-ai/target/release/formal-ai}"
PORT="${PORT:-8911}"
AGENT="${AGENT:-agent}"
TIMEOUT="${TIMEOUT:-120}"
SEED_WORKSPACE="${SEED_WORKSPACE:-}"

CASES="$1"
OUT_ROOT="$2"

VERSION="$("$BIN" --version 2>&1 | head -1)"
# The worktree HEAD moves while siblings commit, so it is *not* the commit the
# binary was built from. Pass BUILD_COMMIT when building, or the record says
# `unknown` rather than implying a provenance it does not have.
BUILD_COMMIT="${BUILD_COMMIT:-unknown}"
COMMIT="$(git -C "$ROOT" rev-parse HEAD)"

SERVER_STATE="$(mktemp -d)"
LOG="/tmp/formal-ai-self-use-$PORT.log"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$SERVER_STATE/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$SERVER_STATE"' EXIT

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on $PORT" >&2
  tail -40 "$LOG" >&2
  exit 1
fi
echo "== server up on $PORT ($VERSION @ $COMMIT) =="

while IFS=$'\t' read -r slug lang prompt; do
  [ -z "${slug:-}" ] && continue
  case "$slug" in \#*) continue ;; esac
  dest="$OUT_ROOT/$slug/$lang"
  mkdir -p "$dest"
  WORKDIR="$(mktemp -d)"
  if [ -n "$SEED_WORKSPACE" ]; then
    cp -R "$SEED_WORKSPACE"/. "$WORKDIR"/ 2>/dev/null
  fi
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
  printf '%s' "$prompt" > "$dest/prompt.txt"
  before="$(grep -c 'POST /v1/chat/completions' "$LOG" 2>/dev/null | head -1)"
  before="${before:-0}"
  log_offset="$(wc -c < "$LOG" 2>/dev/null | tr -d ' ')"
  log_offset="${log_offset:-0}"
  echo "== $slug/$lang =="
  start=$(date +%s)
  # `< /dev/null` is load-bearing: without it the Agent CLI drains this loop's
  # stdin (the cases file) and only the first case ever runs.
  ( cd "$WORKDIR" && timeout "$TIMEOUT" "$AGENT" run \
      --prompt "$prompt" \
      --disable-stdin \
      --no-summarize-session \
      --compaction-models "(same)" \
      --model "formal-ai/formal-ai" < /dev/null ) > "$dest/agent.log" 2>&1
  rc=$?
  end=$(date +%s)
  after="$(grep -c 'POST /v1/chat/completions' "$LOG" 2>/dev/null | head -1)"
  after="${after:-0}"
  # Only this prompt's slice of the server trace, and at most 32 KB of it —
  # `FORMAL_AI_TRACE_REQUESTS=1` echoes whole request bodies, so an untrimmed
  # trace is megabytes per prompt and drowns the transcript it is meant to
  # document. Truncation is stated in the file so no reader mistakes a trimmed
  # trace for a complete one.
  {
    echo "# server trace slice for $slug/$lang, from byte offset $log_offset"
    echo "# truncated to the last 32 KB of the slice if it was longer"
    tail -c "+$((log_offset + 1))" "$LOG" 2>/dev/null | tail -c 32768
  } > "$dest/server-tail.log"
  python3 "$ROOT/docs/case-studies/issue-1138/self-use/extract_answer.py" \
    "$dest/agent.log" "$dest/answer.txt" 2>/dev/null \
    || echo "(extraction failed)" > "$dest/answer.txt"
  ( cd "$WORKDIR" && ls -la ) > "$dest/workspace-listing.txt" 2>&1
  for f in "$WORKDIR"/*; do
    base="$(basename "$f")"
    [ "$base" = "opencode.json" ] && continue
    [ -f "$f" ] && cp "$f" "$dest/workspace-$base" 2>/dev/null
  done
  {
    echo "slug=$slug"
    echo "lang=$lang"
    echo "binary_version=$VERSION"
    echo "binary_built_at_commit=$BUILD_COMMIT"
    echo "worktree_commit_at_run=$COMMIT"
    echo "agent_exit=$rc"
    echo "elapsed_seconds=$((end - start))"
    echo "chat_completion_posts=$((after - before))"
    echo "captured_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } > "$dest/run.env"
  rm -rf "$WORKDIR"
done < "$CASES"

echo "== batch complete =="
