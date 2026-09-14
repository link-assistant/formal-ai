#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #989 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EVIDENCE_ROOT="$ROOT/docs/case-studies/issue-989/self-hosting-authorship"
DECOMPOSITION="issue-989-task-decomposition.lino"
DECOMPOSITION_TASK='Finish Formal AI issue #989 by repairing every dialog regression in the reported session. As one reviewed smallest leaf of that same task, create file issue-989-task-decomposition.lino containing exactly
issue_989_task_decomposition
  total_smallest_leaves 5
  required_self_authored_leaves 1
  leaf dialog_routing author human
  leaf associative_memory_inspection author human
  leaf report_context_links author human
  leaf cross_runtime_regressions author human
  leaf reviewed_task_decomposition author formal_ai'

TASK="$DECOMPOSITION_TASK" \
EXPECT_FILE="$DECOMPOSITION" \
EXPECT_TEXT="required_self_authored_leaves 1" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8989}" \
ARTIFACT_DIR="$EVIDENCE_ROOT/decomposition-session" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$EVIDENCE_ROOT/decomposition-session/$DECOMPOSITION" \
  "$ROOT/docs/case-studies/issue-989/$DECOMPOSITION"
cmp "$EVIDENCE_ROOT/decomposition-session/$DECOMPOSITION" \
  "$ROOT/docs/case-studies/issue-989/$DECOMPOSITION"
grep -m1 -o 'ses_[A-Za-z0-9]*' \
  "$EVIDENCE_ROOT/decomposition-session/agent-cli.log"
