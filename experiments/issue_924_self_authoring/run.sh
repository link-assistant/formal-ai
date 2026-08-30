#!/usr/bin/env bash
# Execute one compound issue #924 self-development task through Formal AI and
# the real Agent CLI. The whole task must fail its exact verifier, split from
# that observed failure, and pass after its smaller authored effects compose.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8924}"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-924/incremental-self-authorship"
WORKSPACE="$(mktemp -d)"
SERVER_LOG="$WORKSPACE/formal-ai.log"
OUTPUT="$WORKSPACE/.formal-ai-orchestration"
TASK='Create file issue-924-coordination.txt containing exactly coordinate issue 924 self-development loop; create file self-development-execution-contract.lino containing exactly
self_development_execution_contract
  record_type "self_development_execution_contract"
  issue "924"
  task_execution "formal_ai_via_agent_cli"
  strategy "attempt_whole_then_split_only_after_failure"
  recursion "split_until_solvable_or_bounded_irreducible"
  effect_application "verified_passing_session_only"
  learning "same_sessions_to_proposal_only_learning"
  promotion "human_review_required"; create file self-development-pull-request-contract.lino containing exactly
self_development_pull_request_contract
  record_type "self_development_pull_request_contract"
  issue "924"
  authorship "end_to_end"
  commit_coverage "every_non_merge_commit_introduced_by_pull_request"
  evidence "session_and_committed_replay_per_commit"
  review_ci_promotion "unchanged"'

cp "$ROOT/experiments/issue_924_self_authoring/verify.sh" "$WORKSPACE/verify.sh"
chmod +x "$WORKSPACE/verify.sh"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKSPACE/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
server_pid=$!
cleanup() {
  local status=$?
  kill "$server_pid" 2>/dev/null || true
  if [ "$status" -eq 0 ]; then
    rm -rf "$WORKSPACE"
  else
    echo "failed workspace preserved at $WORKSPACE" >&2
  fi
}
trap cleanup EXIT

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
  last = trace.fetch("steps").last
  abort "composed parent was not verified" unless last.fetch("passed")
  abort "parent invoked a redundant agent" unless last.fetch("cli") == "composed-verifier"
  abort "learning artifact missing" unless File.file?(ARGV.fetch(2))
' "$WORKSPACE/dispatch-report.json" "$dispatch_status" "$OUTPUT/learning.lino"

mkdir -p "$ARTIFACT_DIR"
rm -rf "$ARTIFACT_DIR/sessions"
cp "$WORKSPACE/self-development-execution-contract.lino" \
  "$ARTIFACT_DIR/self-development-execution-contract.lino"
cp "$WORKSPACE/self-development-pull-request-contract.lino" \
  "$ARTIFACT_DIR/self-development-pull-request-contract.lino"
cp "$WORKSPACE/dispatch-report.json" "$ARTIFACT_DIR/dispatch-report.json"
cp "$OUTPUT/learning.lino" "$ARTIFACT_DIR/learning.lino"
cp "$OUTPUT/proposals.lino" "$ARTIFACT_DIR/proposals.lino"
cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
cp -R "$OUTPUT/sessions" "$ARTIFACT_DIR/sessions"
cp "$WORKSPACE/self-development-execution-contract.lino" \
  "$ROOT/data/meta/self-development-execution-contract.lino"
cp "$WORKSPACE/self-development-pull-request-contract.lino" \
  "$ROOT/data/meta/self-development-pull-request-contract.lino"

grep -h -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR"/sessions/*.json | sed -n '1p'
