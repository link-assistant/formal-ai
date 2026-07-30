#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #703 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-703/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/orchestration-safety-invariant.lino"
TASK='Implement one smallest leaf of Formal AI issue #703. Create file orchestration-safety-invariant.lino containing
orchestration_safety
  record_type meta_invariant
  permission "require an explicit workspace-scoped grant before starting an external agent"
  isolation "run every candidate in a bounded workspace"
  provenance "record process output file effects verification and chained events"
  retry "never retry an external agent implicitly"'

TASK="$TASK" \
EXPECT_FILE="orchestration-safety-invariant.lino" \
EXPECT_TEXT="never retry an external agent implicitly" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8703}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/orchestration-safety-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/orchestration-safety-invariant.lino" "$CANONICAL"
rg -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
