#!/usr/bin/env bash
# Execute one compound issue #933 task through Formal AI and the real Agent
# CLI. The whole task must fail its exact verifier, be split from that failure,
# and pass after its smaller authored leaves have been composed back together.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8933}"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-933/self-hosting-authorship"
WORKSPACE="$(mktemp -d)"
SERVER_LOG="$WORKSPACE/formal-ai.log"
OUTPUT="$WORKSPACE/.formal-ai-orchestration"
TASK='Create file issue-933-coordination.txt containing exactly coordinate issue 933 artifacts; create file variation-floor-contract.lino containing exactly
conversational_variation_floor_contract
  record_type "conversational_variation_floor_contract"
  issue "933"
  minimum_per_language "5"
  languages "en|ru|hi|zh"
  normalization "nfkc_lowercase_strip_punctuation_symbols_separators_whitespace"
  execution "attempt_whole_then_split_on_failure"; create file variation-floor-learning.lino containing exactly
conversational_variation_floor_learning
  record_type "conversational_variation_floor_learning"
  issue "933"
  source "verified_agent_cli_session"
  observation "incremental_dispatch"
  promotion "proposal_only"
  human_review "required"'

cp "$ROOT/experiments/issue_933_self_authoring/verify.sh" "$WORKSPACE/verify.sh"
chmod +x "$WORKSPACE/verify.sh"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKSPACE/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true; rm -rf "$WORKSPACE"' EXIT

curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null

set +e
"$BIN" agent dispatch \
  --incremental \
  --cli agent \
  --task "$TASK" \
  --workspace "$WORKSPACE" \
  --output-dir "$OUTPUT" \
  --base-url "http://127.0.0.1:$PORT" \
  --allow-command bash \
  --verify '["bash","verify.sh"]' \
  > "$WORKSPACE/dispatch-report.json"
dispatch_status=$?
set -e

ruby -rjson -e '
  # The report is UTF-8. `File.read` decodes with the locale default, so a
  # POSIX/C locale turns any non-ASCII byte in a task or verifier message
  # into Encoding::InvalidByteSequenceError and the run reports a failure
  # the dispatch never had. Read the bytes and name their real encoding.
  report = JSON.parse(File.read(ARGV.fetch(0), encoding: Encoding::UTF_8))
  trace = report.fetch("incremental")
  abort "dispatch command failed" unless ARGV.fetch(1) == "0"
  abort "root was not solved" unless trace.fetch("solved")
  abort "whole task unexpectedly passed" if trace.fetch("steps").first.fetch("passed")
  abort "no productive split" unless trace.fetch("splits").any? { |split| split.fetch("children").length >= 2 }
  abort "parent was not retried" unless trace.fetch("steps").last.fetch("passed")
  abort "learning artifact missing" unless File.file?(ARGV.fetch(2))
' "$WORKSPACE/dispatch-report.json" "$dispatch_status" "$OUTPUT/learning.lino"

mkdir -p "$ARTIFACT_DIR"
rm -rf "$ARTIFACT_DIR/sessions"
cp "$WORKSPACE/variation-floor-contract.lino" "$ARTIFACT_DIR/variation-floor-contract.lino"
cp "$WORKSPACE/variation-floor-learning.lino" "$ARTIFACT_DIR/variation-floor-learning.lino"
cp "$WORKSPACE/dispatch-report.json" "$ARTIFACT_DIR/dispatch-report.json"
cp "$OUTPUT/learning.lino" "$ARTIFACT_DIR/learning.lino"
cp "$OUTPUT/proposals.lino" "$ARTIFACT_DIR/proposals.lino"
cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
cp -R "$OUTPUT/sessions" "$ARTIFACT_DIR/sessions"
cp "$WORKSPACE/variation-floor-contract.lino" \
  "$ROOT/data/meta/conversational-variation-floor-contract.lino"

grep -h -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR"/sessions/*.json | sed -n '1p'
