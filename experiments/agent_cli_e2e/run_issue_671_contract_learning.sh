#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8791}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-671/agent-cli-contract-learning}"
OBSERVATIONS="$OUT/observations.jsonl"
EXPECTED="$OUT/client-contract-learning-report.lino"
TASK="Execute the Formal AI client-contract auto-learning task. Run '$BIN clients learn $OBSERVATIONS' and write its exact stdout to client-contract-learning-report.lino. Keep every proposed amendment awaiting human review; do not apply proposals."

command -v "$AGENT" > /dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
[[ -f "$OBSERVATIONS" ]] || {
  echo "missing observations: $OBSERVATIONS" >&2
  exit 2
}
mkdir -p "$OUT"
work="$(mktemp -d)"
completed=0
cleanup() {
  kill "${server_pid:-}" 2> /dev/null || true
  if [[ "$completed" == 1 ]]; then
    rm -rf "$work"
  else
    echo "failed Agent CLI workspace preserved at $work" >&2
  fi
}
trap cleanup EXIT
git -C "$work" init -q

# Keep planning independent of developer memory and background compaction.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" > "$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" > /dev/null
config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"

(cd "$work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  > "$OUT/agent-stream.raw.log" 2> "$OUT/agent-stderr.log")
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" > "$OUT/agent-stream.jsonl"

grep -q 'decision "awaiting_human_review"' \
  "$work/client-contract-learning-report.lino"
grep -q 'client "agent"' "$work/client-contract-learning-report.lino"
cmp "$EXPECTED" "$work/client-contract-learning-report.lino"
cp "$work/client-contract-learning-report.lino" \
  "$OUT/agent-authored-client-contract-learning-report.lino"
cp "$work/.formal-ai/general-change-plan.lino" \
  "$OUT/general-change-plan.lino"
completed=1
echo "issue #671 client-contract learning Agent CLI E2E OK"
