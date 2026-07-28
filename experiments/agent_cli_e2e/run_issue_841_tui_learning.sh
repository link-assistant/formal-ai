#!/usr/bin/env bash
# Formal AI asks a real Agent CLI to derive the same human-gated TUI contract
# report as the deterministic client learner. No proposal is applied.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8795}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-841/tui-contract-learning}"
OBSERVATIONS="${OBSERVATIONS:-$OUT/observations.jsonl}"
EXPECTED="${EXPECTED:-$OUT/tui-contract-learning-report.lino}"
GENERATE_EXPECTED="${GENERATE_EXPECTED:-0}"
REPORT_NAME="tui-contract-learning-report.lino"
TASK="Execute the Formal AI TUI contract auto-learning task. Run '$BIN clients learn $OBSERVATIONS' and write its exact stdout to $REPORT_NAME. Keep every proposed amendment awaiting human review; do not apply proposals."

command -v "$AGENT" > /dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
[[ -f "$OBSERVATIONS" ]] || {
  echo "missing TUI observations: $OBSERVATIONS" >&2
  exit 2
}
mkdir -p "$OUT"
if [[ "$GENERATE_EXPECTED" == 1 ]]; then
  "$BIN" clients learn "$OBSERVATIONS" > "$EXPECTED"
fi
[[ -f "$EXPECTED" ]] || {
  echo "missing expected TUI learning report: $EXPECTED" >&2
  exit 2
}

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

for expected in \
  'decision "awaiting_human_review"' \
  'client "opencode"' \
  'client "claude"' \
  'client "codex"' \
  'field "tui_initial_geometry"' \
  'field "tui_artifact"' \
  'field "tui_renderer_feature"'
do
  grep -q "$expected" "$work/$REPORT_NAME"
done
cmp "$EXPECTED" "$work/$REPORT_NAME"
cp "$work/$REPORT_NAME" "$OUT/agent-authored-$REPORT_NAME"
cp "$work/.formal-ai/general-change-plan.lino" \
  "$OUT/general-change-plan.lino"
completed=1
echo "issue #841 TUI contract learning Agent CLI E2E OK"
