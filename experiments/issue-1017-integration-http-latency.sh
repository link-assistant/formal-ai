#!/usr/bin/env bash
# Issue #1017: measure where an integration-harness HTTP request spends its time.
#
# The macOS core slices failed with `POST should complete: Os { code: 35, kind:
# WouldBlock }` after ~30s, which is exactly the harness `RESPONSE_TIMEOUT`.
# This script separates the three phases the harness measures as one number:
#
#   1. server spawn until `/health` answers,
#   2. the first `/api/openai/v1/chat/completions` POST (cold planner),
#   3. an identical second POST (warm planner).
#
# Usage: experiments/issue-1017-integration-http-latency.sh [concurrency]
#
# With a concurrency argument it runs that many independent server+POST pairs at
# once, which is what `cargo nextest` does on a CI runner: the interesting number
# is the *worst* cold POST, because the harness budget is per-request.
set -euo pipefail

cd "$(dirname "$0")/.."

CONCURRENCY="${1:-1}"
BINARY="target/debug/formal-ai"
TOKEN="sk-local-agentic-tools"

if [[ ! -x "$BINARY" ]]; then
  echo "build the debug binary first: cargo build --all-features" >&2
  exit 1
fi

now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }

one_run() {
  local index="$1"
  local scratch
  scratch="$(mktemp -d)"
  local port
  port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"

  local spawn_start
  spawn_start="$(now_ms)"
  FORMAL_AI_MEMORY_PATH="$scratch/memory.lino" \
    FORMAL_AI_DIALOG_LOG_DIR="$scratch/dialog-logs" \
    FORMAL_AI_API_BEARER_TOKEN="$TOKEN" \
    "$BINARY" serve --host 127.0.0.1 --port "$port" --agent-mode \
    >"$scratch/server.log" 2>&1 &
  local server_pid=$!

  until curl -fsS "http://127.0.0.1:$port/health" >/dev/null 2>&1; do
    if ! kill -0 "$server_pid" 2>/dev/null; then
      echo "run $index: server exited before becoming healthy" >&2
      cat "$scratch/server.log" >&2
      return 1
    fi
    sleep 0.05
  done
  local healthy
  healthy="$(now_ms)"

  local body='{"model":"formal-ai","stream":false,"messages":[{"role":"user","content":"look up the latest news about renewable energy"}],"tools":[{"type":"function","function":{"name":"web_search","parameters":{"type":"object"}}},{"type":"function","function":{"name":"web_fetch","parameters":{"type":"object"}}}]}'

  local post1_start post1_end post2_end
  post1_start="$(now_ms)"
  curl -fsS -X POST "http://127.0.0.1:$port/api/openai/v1/chat/completions" \
    -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
    -d "$body" >"$scratch/response-1.json"
  post1_end="$(now_ms)"
  curl -fsS -X POST "http://127.0.0.1:$port/api/openai/v1/chat/completions" \
    -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
    -d "$body" >"$scratch/response-2.json"
  post2_end="$(now_ms)"

  kill "$server_pid" 2>/dev/null || true
  wait "$server_pid" 2>/dev/null || true

  printf 'run %-3s health %6sms  cold-POST %6sms  warm-POST %6sms\n' \
    "$index" \
    "$((healthy - spawn_start))" \
    "$((post1_end - post1_start))" \
    "$((post2_end - post1_end))"

  rm -rf "$scratch"
}

echo "concurrency=$CONCURRENCY cores=$(getconf _NPROCESSORS_ONLN)"
for index in $(seq 1 "$CONCURRENCY"); do
  one_run "$index" &
done
wait
