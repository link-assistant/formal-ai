#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #905 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-905/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/tool-result-evidence-invariant.lino"
TASK='Fix Formal AI issue #905 by propagating failed tool results, retrying a rejected write once after a read, and allowing completion only after matching verification evidence. As one smallest leaf of that same task, create a file tool-result-evidence-invariant.lino containing exactly:
tool_result_evidence
  record_type meta_invariant
  failure "a failed tool result never completes its planned step"
  retry "after a rejected write read the target then retry once"
  verification "claim completion only when observed evidence matches expected evidence"'

TASK="$TASK" \
EXPECT_FILE="tool-result-evidence-invariant.lino" \
EXPECT_TEXT="observed evidence matches expected evidence" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8845}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/tool-result-evidence-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/tool-result-evidence-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
