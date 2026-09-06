#!/usr/bin/env bash
# Real Agent CLI replay of the issue #1075 sidecar boundary.
#
# The Scala session in the issue ran the client in
# `/tmp/gh-issue-solver-1788563504540` while the server ran under `/home/box`,
# and the served completion asked the client to write
# `/home/box/.formal-ai/general-change-plan.lino`. Agent executed exactly what
# it was given: a real file, on the wrong side of the boundary, invisible to the
# task, and the pull request stayed empty.
#
# `run_agent_cli.sh` cannot show that, because it starts the server inside the
# CLI's own workdir -- server cwd and client cwd are the same directory there,
# so a path rooted in the server's cwd still lands in the right place. This
# script deliberately separates them:
#
#   SIDECAR   the server's working directory   (stands in for /home/box)
#   WORKSPACE the Agent CLI's working directory (stands in for the task clone)
#
# and then requires that everything the run wrote is under WORKSPACE and that
# SIDECAR gained no authored file at all.
#
# Usage: experiments/agent_cli_e2e/run_issue_1075.sh
#
# Environment knobs:
#   BIN       release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT      server port (default: 8975)
#   AGENT     the agent CLI (default: `agent` on PATH)
#   ATTEMPTS  retries for the non-deterministic third-party CLI (default: 3)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8975}"
AGENT="${AGENT:-agent}"
ATTEMPTS="${ATTEMPTS:-3}"
TASK="${TASK:-Write a Rust hello world program to hello.rs, then compile and run it}"
EXPECT_FILE="${EXPECT_FILE:-hello.rs}"

if [ ! -x "$BIN" ]; then
  echo "!! no formal-ai binary at $BIN (cargo build --release --bin formal-ai)" >&2
  exit 1
fi

LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
SIDECAR="$(mktemp -d)"
WORKSPACE="$(mktemp -d)"
SERVER_STATE="$(mktemp -d)"

echo "== sidecar (server cwd): $SIDECAR =="
echo "== workspace (client cwd): $WORKSPACE =="

cat > "$WORKSPACE/opencode.json" <<EOF
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

# The server is started from SIDECAR and never told about WORKSPACE. Everything
# it knows about the client's filesystem has to come out of the requests.
(
  cd "$SIDECAR" || exit 1
  FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$SERVER_STATE/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
  echo $! > "$SERVER_STATE/pid"
)
SRV="$(cat "$SERVER_STATE/pid")"
trap 'kill $SRV 2>/dev/null; rm -rf "$SIDECAR" "$WORKSPACE" "$SERVER_STATE"' EXIT

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on port $PORT"
  tail -60 "$LOG"
  exit 1
fi
echo "== server up on $PORT =="

RC=1
for attempt in $(seq 1 "$ATTEMPTS"); do
  echo "== agent attempt $attempt/$ATTEMPTS =="
  rm -f "$AGENT_LOG"
  (
    cd "$WORKSPACE" || exit 1
    timeout 180 "$AGENT" run \
      --prompt "$TASK" \
      --disable-stdin \
      --no-summarize-session \
      --compaction-model same \
      --model "formal-ai/formal-ai"
  ) > "$AGENT_LOG" 2>&1
  RC=$?
  echo "== agent exit: $RC =="
  [ "$RC" -eq 0 ] && [ -f "$WORKSPACE/$EXPECT_FILE" ] && break
  echo "== attempt $attempt did not produce $EXPECT_FILE; retrying =="
done

echo "== agent tail =="
tail -30 "$AGENT_LOG"

status=0

# 1. The requested file is in the client's workspace.
if [ -f "$WORKSPACE/$EXPECT_FILE" ]; then
  echo "PASS: $EXPECT_FILE written inside the client's workspace"
else
  echo "FAIL: $EXPECT_FILE missing from $WORKSPACE"
  status=1
fi

# 2. Nothing was authored into the server's own directory. This is the exact
#    Scala regression: `.formal-ai/general-change-plan.lino` under the sidecar.
leaked="$(find "$SIDECAR" -type f 2>/dev/null)"
if [ -z "$leaked" ]; then
  echo "PASS: the server's directory gained no authored file"
else
  echo "FAIL: files written into the server's own directory:"
  echo "$leaked"
  status=1
fi

# 3. No emitted absolute path was rooted in the server's directory.
if grep -F "$SIDECAR" "$LOG" > /tmp/issue-1075-sidecar-paths.txt 2>/dev/null; then
  echo "FAIL: the server offered the client paths inside its own directory:"
  head -5 /tmp/issue-1075-sidecar-paths.txt
  status=1
else
  echo "PASS: no emitted path was rooted in the server's directory"
fi

# 4. The plan record, when the recipe keeps one, is the client's file too.
plan="$(find "$WORKSPACE" -name 'general-change-plan.lino' 2>/dev/null | head -1)"
if [ -n "$plan" ]; then
  echo "PASS: the plan record stayed in the client's workspace ($plan)"
fi

echo "== chat rounds: $(grep -c 'POST /v1/chat/completions' "$LOG" 2>/dev/null || echo 0) =="
exit "$status"
