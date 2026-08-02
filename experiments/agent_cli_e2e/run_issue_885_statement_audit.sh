#!/usr/bin/env bash
# Drive the real Agent CLI through Formal AI and audit a whole multi-document
# workspace whose second statement depends on the first.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8895}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-885/agent-cli-evidence/statement-audit}"
FIXTURE="$ROOT/examples/issue-885-relative-audit"
REPORT="statement-audit.lino"
TASK="Fact-check every statement in each Markdown document and the whole workspace, including relative references and dependent probabilities. Use evidence.json as the external evidence capture file and preserve the result in statement-audit.lino."

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
mkdir -p "$OUT"

work="$(mktemp -d)"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT
cp -a "$FIXTURE"/. "$work"/
printf '%s\n' "$TASK" >"$OUT/task.txt"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

agent_config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(
  cd "$work"
  PATH="$(dirname "$BIN"):$PATH" \
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$agent_config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
    --output-format stream-json --compact-json --disable-stdin --prompt "$TASK"
) >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log"

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
session_id="$(grep -Eo '"session_id":"ses_[^"]+' "$OUT/agent-stream.raw.log" | tail -1 | cut -d'"' -f4)"
[[ -n "$session_id" ]] || {
  echo "Agent CLI stream did not preserve a session id" >&2
  exit 1
}
printf '%s\n' "$session_id" >"$OUT/session-id.txt"
[[ -f "$work/$REPORT" ]] || {
  echo "Agent CLI did not write $REPORT" >&2
  exit 1
}
grep -q 'resolved_text "The protocol is independently documented\."' "$work/$REPORT"
grep -q 'contextual_posterior' "$work/$REPORT"
grep -q 'antecedent_statement_id' "$work/$REPORT"
grep -q 'resolution_policy "closest_preceding_subject_same_document"' "$work/$REPORT"
grep -q '^      evidence$' "$work/$REPORT"
grep -q 'evidence_provenance_' "$work/$REPORT"

cp "$work/$REPORT" "$OUT/$REPORT"
rounds="$(grep -c 'POST /' "$OUT/formal-ai.log" || true)"
[[ "$rounds" -ge 2 ]] || {
  echo "expected at least two Agent CLI turns, got $rounds" >&2
  exit 1
}
printf '%s\n' "$rounds" >"$OUT/chat-round-count.txt"
echo "issue 885 Agent CLI document audit passed over $rounds chat rounds: $session_id"
