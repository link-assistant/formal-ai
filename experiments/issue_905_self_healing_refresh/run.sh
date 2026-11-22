#!/usr/bin/env bash
# Refresh the planner-derived repair fixture through Formal AI and Agent CLI.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-905/self-hosting-fixture-refresh"
TASK="$(cat "$ROOT/rust/tests/fixtures/issue-558/repair-request.md")"

TASK="$TASK" \
EXPECT_FILE="self-healing-case.lino" \
EXPECT_TEXT='benchmark_status "absent"' \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8846}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

grep -F 'module_path "src/event_log.rs"' "$ARTIFACT_DIR/self-healing-case.lino"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
