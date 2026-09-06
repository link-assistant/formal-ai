#!/usr/bin/env bash
# Real Agent CLI replay of the issue #1075 sidecar boundary.
#
# The Scala session ran the client in `/tmp/gh-issue-solver-1788563504540` while
# the server ran under `/home/box`, and the served completion asked the client to
# write `/home/box/.formal-ai/general-change-plan.lino`. Agent executed exactly
# what it was given: a real file, on the wrong side of the boundary, invisible to
# the task, and the pull request stayed empty.
#
# Reproducing that needs more than two different directories. The pre-fix
# `is_usable_directory` followed a declared workspace whenever the server could
# `is_dir()` it, so with both processes on one host the declaration was honoured
# and nothing went wrong. What made the session fail is that the server *could
# not see* the client's directory -- they were different containers. So the
# server here runs in a container with only its own sidecar mounted, and the
# client's workspace genuinely does not exist from where the server stands.
#
#   SIDECAR   the server's working directory, mounted at /sidecar in the container
#   WORKSPACE the Agent CLI's working directory, on the host, invisible to the server
#
# Everything the run wrote must be under WORKSPACE, and SIDECAR must gain no
# authored file at all.
#
# Usage: experiments/agent_cli_e2e/run_issue_1075.sh
#
# Environment knobs:
#   BIN       release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT      server port (default: 8975)
#   AGENT     the agent CLI (default: `agent` on PATH)
#   IMAGE     container base with a matching glibc (default: ubuntu:24.04)
#   ATTEMPTS  retries for the non-deterministic third-party CLI (default: 3)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8975}"
AGENT="${AGENT:-agent}"
IMAGE="${IMAGE:-ubuntu:24.04}"
ATTEMPTS="${ATTEMPTS:-3}"
# The phrasing the release matrix already uses for this task, so the run
# exercises the recognised recipe rather than a paraphrase of it.
TASK="${TASK:-Give me hello world program in Rust}"
EXPECT_FILE="${EXPECT_FILE:-main.rs}"
# The greeting the seed actually carries. `data/seed/hello-world-programs.lino`
# spells it with a lowercase `w` in every language it lists, and the release
# matrix asserts the same string, so that is what a correct run prints.
EXPECT_GREETING="${EXPECT_GREETING:-Hello, world!}"

if [ ! -x "$BIN" ]; then
  echo "!! no formal-ai binary at $BIN (cargo build --release --bin formal-ai)" >&2
  exit 1
fi
if ! docker info >/dev/null 2>&1; then
  echo "!! docker is required: the server has to run where the client's directory does not exist" >&2
  exit 1
fi

LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
SIDECAR="$(mktemp -d)"
WORKSPACE="$(mktemp -d)"
STATE="$(mktemp -d)"
NAME="formal-ai-issue-1075-$PORT"

echo "== sidecar (server cwd, in a container): $SIDECAR -> /sidecar =="
echo "== workspace (client cwd, on the host): $WORKSPACE =="

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

cleanup() {
  docker rm -f "$NAME" >/dev/null 2>&1
  rm -rf "$SIDECAR" "$WORKSPACE" "$STATE"
}
trap cleanup EXIT

# The server is started in a container whose filesystem contains /sidecar and
# not $WORKSPACE. Everything it can know about the client's directory has to
# come out of the requests, because it cannot look.
docker run --rm --name "$NAME" --network host \
  --user "$(id -u):$(id -g)" \
  -v "$BIN:/usr/local/bin/formal-ai:ro" \
  -v "$SIDECAR:/sidecar" \
  -v "$STATE:/state" \
  -w /sidecar \
  -e FORMAL_AI_AGENT_MODE=1 -e FORMAL_AI_TRACE_REQUESTS=1 \
  -e FORMAL_AI_MEMORY_PATH=/state/memory.lino -e FORMAL_AI_DREAMING=0 \
  "$IMAGE" formal-ai serve --host 0.0.0.0 --port "$PORT" > "$LOG" 2>&1 &

if ! curl -sS --retry 40 --retry-delay 1 --retry-connrefused --max-time 60 \
     "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  echo "!! server never came up on port $PORT"
  tail -60 "$LOG"
  exit 1
fi
echo "== server up on $PORT, inside $NAME =="

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

status=0

# The server's own memory lives in /state, mounted separately, so anything under
# /sidecar is an authored file and nothing else.
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

# 3. No emitted absolute path was rooted in the server's own directory. Inside
#    the container that directory is /sidecar, which is what the client would be
#    handed.
if grep -Eo '"[^"]*/sidecar/[^"]*"' "$LOG" > /tmp/issue-1075-sidecar-paths.txt 2>/dev/null; then
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

# 5. The program is a program: it compiles and prints the seeded greeting. The
#    issue's sessions all produced a well-formed *call* and no working artifact,
#    so existence of the file is not the claim being made here.
if [ -f "$WORKSPACE/$EXPECT_FILE" ] && command -v rustc >/dev/null 2>&1; then
  if (cd "$WORKSPACE" && rustc -O -o hello-bin "$EXPECT_FILE" > /tmp/issue-1075-rustc.log 2>&1); then
    greeting="$("$WORKSPACE/hello-bin" 2>&1)"
    if [ "$greeting" = "$EXPECT_GREETING" ]; then
      echo "PASS: the program compiles and prints exactly: $greeting"
    else
      echo "FAIL: the program printed '$greeting', expected '$EXPECT_GREETING'"
      status=1
    fi
  else
    echo "FAIL: $EXPECT_FILE does not compile"
    head -20 /tmp/issue-1075-rustc.log
    status=1
  fi
fi

echo "== chat rounds: $(grep -c 'POST /v1/chat/completions' "$LOG" 2>/dev/null || echo 0) =="
echo "== agent tail =="
tail -12 "$AGENT_LOG"
exit "$status"
