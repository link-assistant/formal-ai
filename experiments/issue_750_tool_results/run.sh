#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8788}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-750/agent-cli-evidence}"
WORK="${WORK:-$(mktemp -d)}"
SERVER_PID=

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"
git -C "$WORK" init -q
git -C "$WORK" config user.email issue-750@example.invalid
git -C "$WORK" config user.name issue-750-fixture
touch "$WORK/alpha.txt" "$WORK/beta.json"
git -C "$WORK" add alpha.txt beta.json
git -C "$WORK" commit -qm fixture

FORMAL_AI_AGENT_MODE=1 \
FORMAL_AI_TRACE_REQUESTS=1 \
FORMAL_AI_MEMORY_PATH="$OUT/memory.lino" \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
SERVER_PID=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(cd "$WORK" && \
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
    --output-format stream-json --compact-json --disable-stdin \
    --prompt "Run ls to list files here" \
    >"$OUT/agent-stream.jsonl" 2>"$OUT/agent-stderr.log")

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
node "$ROOT/experiments/issue_750_tool_results/extract-final.mjs" \
  "$OUT/agent-stream.jsonl" "$OUT/final-answer.txt"
rg -F "alpha.txt" "$OUT/final-answer.txt"
rg -F "beta.json" "$OUT/final-answer.txt"
if rg -q 'untrusted_context|Process Group PGID|"exit_code"' "$OUT/final-answer.txt"; then
  echo "transport envelope leaked into final answer" >&2
  exit 1
fi

cat "$OUT/final-answer.txt"
echo "Agent CLI evidence passed; disposable fixture retained at $WORK"
