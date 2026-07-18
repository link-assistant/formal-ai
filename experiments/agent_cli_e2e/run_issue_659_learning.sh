#!/usr/bin/env bash
# Execute issue #659's associative auto-learning task through two real Agent
# CLIs, with Formal AI itself serving as each CLI's model provider.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OPENCODE="${OPENCODE:-opencode}"
PORT="${PORT:-8882}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-659/agent-cli-evidence}"
REPORT="hardcoded-language-learning-report.lino"
TASK='Reconsider the persisted R379 source-language audit as an associative evidence network, rank what it teaches about future amendments, leave adoption to a reviewer, and save hardcoded-language-learning-report.lino.'

command -v "$AGENT" >/dev/null
command -v "$OPENCODE" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"

agent_work="$(mktemp -d)"
opencode_work="$(mktemp -d)"
state="$(mktemp -d)"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$agent_work" "$opencode_work" "$state"
}
trap cleanup EXIT

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

assert_report() {
  local harness="$1" file="$2"
  [[ -f "$file" ]] || { echo "$harness: $REPORT was never written" >&2; exit 1; }
  grep -q '^hardcoded_language_learning_report$' "$file"
  grep -q 'issue "659"' "$file"
  grep -q 'decision "awaiting_human_review"' "$file"
  grep -q 'promotion_gate "hardcoded_language_fixture_context_gate_and_agent_cli_e2e_pass"' "$file"
  grep -q 'observation:sentence-only-gap' "$file"
  grep -q 'observation:seed-duplication' "$file"
  grep -q 'lesson:context-sensitive-detection' "$file"
  grep -q 'lesson:two-way-ratchet' "$file"
  grep -q 'lesson:seed-first-migration' "$file"
  if grep -q 'decision "promoted"' "$file"; then
    echo "$harness: report promoted itself without human review" >&2
    exit 1
  fi
  local ranked
  ranked="$(grep -c '^  learned_expression_' "$file")"
  [[ "$ranked" -ge 10 ]] || {
    echo "$harness: expected all persisted evidence to be ranked, found $ranked" >&2
    exit 1
  }
  echo "$harness: report OK ($ranked ranked expressions, $(wc -c <"$file") bytes)"
}

echo "== harness 1/2: @link-assistant/agent =="
agent_config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
git -C "$agent_work" init -q
(cd "$agent_work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$agent_config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
assert_report "agent" "$agent_work/$REPORT"

echo "== harness 2/2: opencode =="
cat >"$opencode_work/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  },
  "model": "formal-ai/formal-ai"
}
EOF
(cd "$opencode_work" && XDG_DATA_HOME="$state/data" XDG_CONFIG_HOME="$state/config" \
  XDG_CACHE_HOME="$state/cache" \
  "$OPENCODE" run --pure --auto --format json --model formal-ai/formal-ai "$TASK" \
  >"$OUT/opencode-stream.jsonl" 2>"$OUT/opencode-stderr.log")
assert_report "opencode" "$opencode_work/$REPORT"

if ! diff -u "$agent_work/$REPORT" "$opencode_work/$REPORT" >"$OUT/harness-parity.diff"; then
  echo "the two harnesses derived different reports; see $OUT/harness-parity.diff" >&2
  exit 1
fi
rm -f "$OUT/harness-parity.diff"
cp "$agent_work/$REPORT" "$OUT/$REPORT"

rounds="$(grep -c 'POST /' "$OUT/formal-ai.log" || true)"
echo "issue #659 Agent CLI E2E OK: two harnesses derived a byte-identical, review-gated report over $rounds chat rounds"
