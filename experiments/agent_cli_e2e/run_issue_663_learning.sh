#!/usr/bin/env bash
set -euo pipefail

# Issue #663 — Formal AI re-derives the specialized-handler precedence itself.
# Formal AI serves as the model provider (no external model, no API key) while
# the real @link-assistant/agent CLI executes a differently-worded version of
# the learning task and must author the review-gated handler-precedence report.

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8885}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-663/agent-cli-evidence}"
TASK='Have Formal AI learn from its own history: read the persisted issue 663 specialized-handler precedence rationale as an associative links network, rank the ordering observations and the precedence-is-data amendment, keep the proposal gated on human review, and save it to handler-precedence-learning-report.lino.'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"
work="$(mktemp -d)"
cleanup() { kill "${server_pid:-}" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
git -C "$work" init -q

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
config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"

(cd "$work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"

report="$work/handler-precedence-learning-report.lino"
grep -q 'decision "awaiting_human_review"' "$report"
grep -q 'lesson:precedence-is-data' "$report"
grep -q 'promotion_gate "routing_precedence_from_seed_and_parity_fixture_pass"' "$report"

# The Agent-CLI-authored artifact must match the byte-for-byte committed
# evidence that `cargo test committed_agent_cli_artifact_is_byte_reproducible`
# also pins, so the tool — not a hand-edit — is the author.
diff -u "$OUT/handler-precedence-learning-report.lino" "$report"
echo "issue #663 handler-precedence-learning Agent CLI E2E OK"
