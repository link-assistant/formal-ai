#!/usr/bin/env bash
# Boot formal-ai serve, wait for health, run a curl against it, tear down.
# All in one process tree so the environment does not reap the server.
#
# Sibling of run_agent_cli.sh: this one skips the agent CLI and just POSTs a
# hand-written /v1/chat/completions payload — useful when isolating server-side
# behaviour (SSE framing, tool_call planner, permission gate) from the CLI.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8734}"
LOG="/tmp/formal-ai-serve-$PORT.log"
MEMDIR="$(mktemp -d)"

# Private, empty memory so this diagnostic server observes server-side behaviour
# from a clean seed rather than whatever the shared ~/.formal-ai/memory.lino
# accumulated (issue #828); FORMAL_AI_DREAMING=0 stops background compaction.
FORMAL_AI_AGENT_MODE=1 \
  FORMAL_AI_MEMORY_PATH="$MEMDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$MEMDIR"' EXIT

# Wait for readiness without foreground sleep (curl retry handles the backoff).
curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 || { echo "server never came up"; cat "$LOG"; exit 1; }
echo "== server up =="

curl -sS "http://127.0.0.1:$PORT/v1/chat/completions" \
  -H 'content-type: application/json' -H 'authorization: Bearer x' \
  -d '{"model":"formal-ai","messages":[{"role":"user","content":"Create a file hi.txt with content hello"}],"tools":[{"type":"function","function":{"name":"write","description":"Write a file","parameters":{"type":"object","properties":{"filePath":{"type":"string"},"content":{"type":"string"}}}}}]}'
echo
