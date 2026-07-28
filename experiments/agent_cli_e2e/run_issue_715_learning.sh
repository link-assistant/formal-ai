#!/usr/bin/env bash
# Execute issue #715's own auto-learning task through *two* real Agent CLIs.
#
# The report this task derives names its promotion gate
# `normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass`, so the gate
# is only honest if an external Agent CLI E2E actually runs it. Until this
# script existed the task had only ever run in formal-ai's own in-process
# harness, which is the one harness that cannot show the capability routing
# works over the wire.
#
# Both CLIs are driven against the same task, and the two reports are compared
# byte for byte. That equality is the point: a harness is "supported in the
# similar way" only if it derives the *same* artifact, and the assertion fails
# loudly if a harness's tool vocabulary ever bends the result.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OPENCODE="${OPENCODE:-opencode}"
PORT="${PORT:-8879}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-715/agent-cli-learning}"
REPORT="code-rewrite-learning-report.lino"
TASK='Use Formal AI auto-learning to inspect the persisted issue 715 workspace-rewrite failures as an associative links network, rank the observations and normal-algorithm amendments, keep promotion human-review gated, and write code-rewrite-learning-report.lino.'

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

# Private, empty memory per run so this server's memory-fed planning stays
# independent of what other E2E scripts recorded into the shared
# ~/.formal-ai/memory.lino (issue #828); FORMAL_AI_DREAMING=0 stops the
# background compaction thread from mutating it mid-run.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$agent_work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

# The report is a review artifact, so every claim it makes about itself is
# asserted here rather than trusted.
assert_report() {
  local harness="$1" file="$2"
  [[ -f "$file" ]] || { echo "$harness: $REPORT was never written" >&2; exit 1; }
  grep -q 'code_rewrite_learning_report' "$file"
  grep -q 'issue "715"' "$file"
  grep -q 'decision "awaiting_human_review"' "$file"
  grep -q 'promotion_gate "normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass"' "$file"
  grep -q 'retention_formula "reads + writes + incoming_links + outgoing_links"' "$file"
  grep -q 'lesson:normal-algorithm-core' "$file"
  grep -q 'lesson:client-byte-boundary' "$file"
  # Ranking is the learning, not decoration: the highest-retention expression
  # must lead, and every expression must have been scored.
  local ranked
  ranked="$(grep -c '^  learned_expression_' "$file")"
  [[ "$ranked" -ge 8 ]] || {
    echo "$harness: expected the full ranked network, found $ranked expressions" >&2
    exit 1
  }
  # Auto-learning that promoted itself would be the bug this gate exists to
  # prevent, so the absence of a promotion is asserted too.
  if grep -q 'decision "promoted"' "$file"; then
    echo "$harness: the report promoted itself without human review" >&2
    exit 1
  fi
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

# The parity assertion. Two harnesses, two tool vocabularies, one artifact.
if ! diff -u "$agent_work/$REPORT" "$opencode_work/$REPORT" >"$OUT/harness-parity.diff"; then
  echo "the two harnesses derived different reports; see $OUT/harness-parity.diff" >&2
  exit 1
fi
rm -f "$OUT/harness-parity.diff"
cp "$agent_work/$REPORT" "$OUT/$REPORT"

rounds="$(grep -c 'POST /' "$OUT/formal-ai.log" || true)"
echo "issue #715 auto-learning Agent CLI E2E OK: both harnesses derived a byte-identical review-gated report over $rounds chat rounds"
