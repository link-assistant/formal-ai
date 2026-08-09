#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #873 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-873/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/research-learning-recovery-invariant.lino"
TASK='Finish Formal AI issue #873 by making every unknown a research frontier and by preserving a tested stable memory version through learning and recovery. As one smallest leaf of that same task, create file research-learning-recovery-invariant.lino containing
research_learning_recovery_invariant
  record_type meta_invariant
  unknown "inspect local state then research reachable external sources"
  evidence "retain source provenance while allowing recomputable payload eviction"
  promotion "activate a candidate only after the immutable baseline passes"
  recovery "keep the previous tested stable version active after any candidate failure"
  autonomy "ask on ambiguity rank recorded outcomes in full trust and gate each command when configured"
  budget "after the configured limit preserve the current plan and request continuation"'

TASK="$TASK" \
EXPECT_FILE="research-learning-recovery-invariant.lino" \
EXPECT_TEXT="preserve the current plan and request continuation" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8873}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/research-learning-recovery-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/research-learning-recovery-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
