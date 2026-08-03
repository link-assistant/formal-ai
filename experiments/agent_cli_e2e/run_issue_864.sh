#!/usr/bin/env bash
# Real Agent CLI ↔ formal-ai regression for proactive failure reporting (#864).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8864}"
OUT="${OUT:-/tmp/formal-ai-issue-864-evidence}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue-864.XXXXXX")"
SERVER_PID=

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}

mkdir -p "$OUT"
git -C "$WORK" init -q
git -C "$WORK" config user.email issue-864@example.invalid
git -C "$WORK" config user.name issue-864-fixture
printf '%s\n' '# Failure-report fixture' >"$WORK/README.md"
git -C "$WORK" add README.md
git -C "$WORK" commit -qm fixture

TASK="Run issue_864_command_that_does_not_exist"
printf '%s\n' "$TASK" >"$OUT/task.txt"

FORMAL_AI_AGENT_MODE=1 \
FORMAL_AI_TRACE_REQUESTS=1 \
FORMAL_AI_MEMORY_PATH="$WORK/memory.lino" \
FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" \
  >"$OUT/formal-ai.log" 2>&1 &
SERVER_PID=$!

curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(
  printf \
    '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' \
    "$PORT"
)"

(
  cd "$WORK"
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" \
    --model formalai/formal-ai \
    --permission-mode auto \
    --output-format stream-json \
    --compact-json \
    --disable-stdin \
    --prompt "$TASK"
) >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log"

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
rg '^\{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
node "$ROOT/experiments/issue_750_tool_results/extract-final.mjs" \
  "$OUT/agent-stream.jsonl" "$OUT/final-answer.txt"

rg -F 'The command failed:' "$OUT/final-answer.txt"
rg -F 'Would you like me to prepare an issue report with the diagnostic context?' \
  "$OUT/final-answer.txt"
rg -F '`Report issue`' "$OUT/final-answer.txt"
rg -F 'agentic_outcome: planned ToolCalls' "$OUT/formal-ai.log"
rg -F 'arguments: "{\"command\":\"issue_864_command_that_does_not_exist\"}"' \
  "$OUT/formal-ai.log"
rg -F 'agentic_outcome: planned Final("The command failed:' "$OUT/formal-ai.log"
if rg -q 'gh issue create' "$OUT/formal-ai.log" "$OUT/agent-stream.raw.log"; then
  echo "the opt-in invitation filed an issue without user confirmation" >&2
  exit 1
fi

posts="$(rg -c 'POST /api/openai/v1/chat/completions' "$OUT/formal-ai.log")"
[[ "$posts" -ge 2 ]] || {
  echo "expected a tool-call round trip, got $posts chat completion(s)" >&2
  exit 1
}

echo "issue 864 Agent CLI E2E passed: failed command invited an opt-in report ($posts rounds)"
