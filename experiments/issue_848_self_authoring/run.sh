#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #848 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-848/self-hosting-authorship"
TASK='Advance Formal AI issue #848 by executing a growing ladder of real coding tasks through formal-ai with agent, scoring only observed workspace effects, and preserving each verified result. As one smallest leaf of that same task, create file coding-task-execution-invariant.lino containing
coding_task_execution_contract
  record_type meta_invariant
  source_generation "render executable source from the formalized request, never echo request prose"
  verification "compile or inspect the exact bytes written to the requested workspace target"
  scoring "record success only after an observed_workspace_effect passes its task verifier"'

TASK="$TASK" \
EXPECT_FILE="coding-task-execution-invariant.lino" \
EXPECT_TEXT="record success only after an observed_workspace_effect passes its task verifier" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8848}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

grep -q 'source_generation "render executable source' \
  "$ARTIFACT_DIR/coding-task-execution-invariant.lino"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
