#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #847 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-847/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/task-decomposition-invariant.lino"
TASK='Deepen Formal AI issue #847 by making issue-sized tasks descend through general reviewed strategies, reusing the exact inspected tree for recursive execution, and learning only through green regression evidence plus human review. As one smallest leaf of that same task, create file task-decomposition-invariant.lino containing
task_decomposition_contract
  record_type meta_invariant
  atomic "require an observable completion contract and no pending children"
  execution "run the exact inspected content-addressed tree"
  learning "activate a strategy only after green regression evidence and human review"'

TASK="$TASK" \
EXPECT_FILE="task-decomposition-invariant.lino" \
EXPECT_TEXT="run the exact inspected content-addressed tree" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8847}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/task-decomposition-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/task-decomposition-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
