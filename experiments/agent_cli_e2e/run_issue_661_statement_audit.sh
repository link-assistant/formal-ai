#!/usr/bin/env bash
# Execute issue #661's repository audit through the real @link-assistant/agent
# CLI, with Formal AI serving as its model provider and the release binary
# executing the audit in the client-owned workspace.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8884}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-661/agent-cli-evidence}"
FIXTURE="$ROOT/examples/issue-661-statement-audit"
REPORT="statement-audit.lino"
TASK="Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino."
COMMAND="formal-ai statement-audit --root . --output statement-audit.lino"

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

# Private, empty memory per run so this server's memory-fed planning stays
# independent of what other E2E scripts recorded into the shared
# ~/.formal-ai/memory.lino (issue #828); FORMAL_AI_DREAMING=0 stops the
# background compaction thread from mutating it mid-run.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

agent_config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(cd "$work" && \
  PATH="$(dirname "$BIN"):$PATH" \
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$agent_config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
    --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
    >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"

[[ -f "$work/$REPORT" ]] || {
  echo "@link-assistant/agent did not write $REPORT" >&2
  exit 1
}
grep -q '^repository_statement_audit$' "$work/$REPORT"
grep -q 'type "requirement_contradiction"' "$work/$REPORT"
grep -q 'type "audit_finding"' "$work/$REPORT"
grep -q 'disposition "issue_candidate"' "$work/$REPORT"
grep -q 'relative_weight' "$work/$REPORT"
grep -q 'associations' "$work/$REPORT"
grep -Fq "$COMMAND" "$OUT/formal-ai.log"

rounds="$(grep -c 'POST /' "$OUT/formal-ai.log" || true)"
[[ "$rounds" -ge 2 ]] || {
  echo "expected at least two Agent CLI turns, got $rounds" >&2
  exit 1
}

cp "$work/$REPORT" "$OUT/$REPORT"
echo "issue #661 Agent CLI E2E OK: audit produced over $rounds chat rounds"
