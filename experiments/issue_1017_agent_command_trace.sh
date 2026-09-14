#!/usr/bin/env bash
# Issue #1017: demonstrate the off-by-default `FORMAL_AI_TRACE_COMMANDS` verbose mode.
#
# A macOS core slice failed with `timed_out: true` on a `python3` command, and the
# log said only that a deadline was reached — not which binary ran, nor for how
# long. `FORMAL_AI_TRACE_COMMANDS=1` reports both, so the next occurrence on a
# runner nobody can attach a debugger to identifies itself.
#
# Usage: bash experiments/issue_1017_agent_command_trace.sh
set -euo pipefail

cd "$(dirname "$0")/.."

TEST=agent::tests::python3_command_runs_from_allowlisted_resolved_path

echo "== default (trace off) =="
{ cargo test --test source --all-features -- --nocapture "$TEST" 2>&1 || true; } |
  { grep -c '\[agent-command\]' || true; } |
  sed 's/^/[agent-command] lines: /'

echo
echo "== FORMAL_AI_TRACE_COMMANDS=1 =="
FORMAL_AI_TRACE_COMMANDS=1 \
  cargo test --test source --all-features -- --nocapture "$TEST" 2>&1 |
  grep '\[agent-command\]' || true
