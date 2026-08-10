#!/usr/bin/env bash
# Real Agent CLI asks Formal AI to derive the #709 report from associative memory.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT="$ROOT/docs/case-studies/issue-709/agent-cli-evidence/learning-report-execution"
TASK='Use Formal AI auto-learning to inspect the persisted issue 709 search-fusion execution failures and corrections as an associative links network, rank the reusable amendments, keep recipe promotion human-review gated, and write search-fusion-learning-report.lino.'

TASK="$TASK" EXPECT_FILE="search-fusion-learning-report.lino" \
  EXPECT_TEXT="lesson:gated-recipe-replay" MIN_POSTS=3 ATTEMPTS=3 PORT="${PORT:-8714}" \
  BIN="${BIN:-$ROOT/target/debug/formal-ai}" ARTIFACT_DIR="$ARTIFACT" \
  "$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

grep -q 'decision "awaiting_human_review"' "$ARTIFACT/search-fusion-learning-report.lino"
grep -q 'promotion_gate "issue_709_held_out_zero_failures_and_named_review"' \
  "$ARTIFACT/search-fusion-learning-report.lino"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT/agent-cli.log"
