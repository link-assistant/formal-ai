#!/usr/bin/env bash
# Refresh the planner-derived repair fixture through Formal AI and Agent CLI.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-905/self-hosting-fixture-refresh"
CANONICAL="$ROOT/data/meta/self-healing-case.lino"
TASK='When you cannot answer an input, run your self-healing loop: reason about the failure, map it onto the source that would change with a source-to-links round-trip, learn a benchmark-gated lesson, and record the repair case in Links Notation for human approval.'

TASK="$TASK" \
EXPECT_FILE="self-healing-case.lino" \
EXPECT_TEXT='total_link_count "13100"' \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8846}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cmp "$ARTIFACT_DIR/self-healing-case.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
