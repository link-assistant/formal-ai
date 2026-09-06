#!/usr/bin/env bash
# The issue #1075 sidecar boundary, replayed through the real coding CLIs.
#
# The Scala session ran the client in `/tmp/gh-issue-solver-1788563504540` while
# the server ran under `/home/box`, and the served completion asked the client to
# write `/home/box/.formal-ai/general-change-plan.lino`. The client executed
# exactly what it was given: a real file, on the wrong side of the boundary,
# invisible to the task, and the pull request stayed empty.
#
# Reproducing that needs more than two different directories. The pre-fix
# `is_usable_directory` followed a declared workspace whenever the server could
# `is_dir()` it, so with both processes on one host the declaration was honoured
# and nothing went wrong. What made the session fail is that the server *could
# not see* the client's directory -- they were different containers. So the
# server here runs in a container with only its own sidecar mounted, and each
# client's workspace genuinely does not exist from where the server stands.
#
#   SIDECAR   the server's working directory, mounted at /sidecar in the container
#   WORKSPACE a fresh host directory per client, invisible to the server
#
# Each client must leave the program in its own workspace, leave the sidecar
# empty, and produce something that compiles and prints the seeded greeting.
#
# Usage: experiments/agent_cli_e2e/run_issue_1075.sh
#
# Environment knobs:
#   BIN       release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT      server port (default: 8975)
#   CLIENTS   which CLIs to drive (default: "agent claude")
#   IMAGE     container base with a matching glibc (default: ubuntu:24.04)
#   ATTEMPTS  retries for the non-deterministic third-party CLIs (default: 3)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8975}"
IMAGE="${IMAGE:-ubuntu:24.04}"
ATTEMPTS="${ATTEMPTS:-3}"
CLIENTS="${CLIENTS:-agent claude}"
# The phrasing the release matrix already uses for this task, so the run
# exercises the recognised recipe rather than a paraphrase of it
# (.github/workflows/release.yml:1091).
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
SIDECAR="$(mktemp -d)"
STATE="$(mktemp -d)"
NAME="formal-ai-issue-1075-$PORT"
WORKSPACES=""

cleanup() {
  docker rm -f "$NAME" >/dev/null 2>&1
  # shellcheck disable=SC2086
  rm -rf "$SIDECAR" "$STATE" $WORKSPACES
}
trap cleanup EXIT

echo "== sidecar (server cwd, in a container): $SIDECAR -> /sidecar =="

# The server runs in a container whose filesystem contains /sidecar and none of
# the client workspaces. Everything it can know about a client's directory has
# to come out of that client's requests, because it cannot look.
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

status=0

# Drive one client in a fresh workspace of its own. Each gets the same task and
# is judged on the same four things.
drive() {
  client="$1"
  if ! command -v "$client" >/dev/null 2>&1; then
    echo "SKIP: $client is not installed"
    return 0
  fi

  workspace="$(mktemp -d)"
  WORKSPACES="$WORKSPACES $workspace"
  log="/tmp/issue-1075-$client-$PORT.log"
  echo
  echo "== $client: workspace (client cwd, on the host): $workspace =="

  case "$client" in
    agent)
      cat > "$workspace/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": { "baseURL": "http://127.0.0.1:$PORT/v1", "apiKey": "local" },
      "models": { "formal-ai": { "name": "Formal AI Symbolic Production" } }
    }
  }
}
EOF
      ;;
  esac

  for attempt in $(seq 1 "$ATTEMPTS"); do
    echo "== $client attempt $attempt/$ATTEMPTS =="
    case "$client" in
      agent)
        ( cd "$workspace" && timeout 180 agent run --prompt "$TASK" \
            --disable-stdin --no-summarize-session --compaction-model same \
            --model "formal-ai/formal-ai" ) > "$log" 2>&1
        ;;
      claude)
        # Claude Code speaks the Anthropic Messages API, served at
        # /api/anthropic/v1/messages (README "Claude Code").
        ( cd "$workspace" && ANTHROPIC_BASE_URL="http://127.0.0.1:$PORT/api/anthropic" \
            ANTHROPIC_API_KEY="local-test-token" \
            timeout 180 claude -p "$TASK" --permission-mode bypassPermissions \
              --model formal-ai ) > "$log" 2>&1
        ;;
      *)
        echo "!! unknown client $client"
        return 1
        ;;
    esac
    echo "== $client exit: $? =="
    [ -f "$workspace/$EXPECT_FILE" ] && break
    echo "== attempt $attempt did not produce $EXPECT_FILE; retrying =="
  done

  # 1. The requested file is in the client's own workspace.
  if [ -f "$workspace/$EXPECT_FILE" ]; then
    echo "PASS[$client]: $EXPECT_FILE written inside the client's workspace"
  else
    echo "FAIL[$client]: $EXPECT_FILE missing from $workspace"
    status=1
  fi

  # 2. The program is a program. Every session in the issue produced a
  #    well-formed call and no working artifact, so existence is not the claim.
  if [ -f "$workspace/$EXPECT_FILE" ] && command -v rustc >/dev/null 2>&1; then
    if ( cd "$workspace" && rustc -O -o issue-1075-bin "$EXPECT_FILE" ) \
         > "/tmp/issue-1075-rustc-$client.log" 2>&1; then
      greeting="$("$workspace/issue-1075-bin" 2>&1)"
      if [ "$greeting" = "$EXPECT_GREETING" ]; then
        echo "PASS[$client]: the program compiles and prints exactly: $greeting"
      else
        echo "FAIL[$client]: the program printed '$greeting', expected '$EXPECT_GREETING'"
        status=1
      fi
    else
      echo "FAIL[$client]: $EXPECT_FILE does not compile"
      head -20 "/tmp/issue-1075-rustc-$client.log"
      status=1
    fi
  fi
}

for client in $CLIENTS; do
  drive "$client"
done

echo

# 3. Nothing was authored into the server's own directory. This is the exact
#    Scala regression: `.formal-ai/general-change-plan.lino` under the sidecar.
#    The server's own memory lives in /state, mounted separately, so anything
#    under /sidecar is an authored file and nothing else.
leaked="$(find "$SIDECAR" -type f 2>/dev/null)"
if [ -z "$leaked" ]; then
  echo "PASS: the server's directory gained no authored file"
else
  echo "FAIL: files written into the server's own directory:"
  echo "$leaked"
  status=1
fi

# 4. No absolute path the server emitted was rooted in its own directory. Inside
#    the container that directory is /sidecar, which is what a client is handed.
if grep -Eo '"[^"]*/sidecar/[^"]*"' "$LOG" | sort -u > /tmp/issue-1075-sidecar-paths.txt \
   && [ -s /tmp/issue-1075-sidecar-paths.txt ]; then
  echo "FAIL: the server offered clients paths inside its own directory:"
  head -8 /tmp/issue-1075-sidecar-paths.txt
  status=1
else
  echo "PASS: no emitted path was rooted in the server's directory"
fi

echo "== chat rounds: $(grep -c 'POST /' "$LOG" 2>/dev/null || echo 0) =="
exit "$status"
