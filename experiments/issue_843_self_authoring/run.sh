#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #843 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-843/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/source-evidence-honesty-invariant.lino"
TASK='Fix Formal AI issue #843 by replacing fabricated source and cache evidence with exact captured bytes, deterministic offline replay, real provider rankings, and human-gated auto-learning. As one smallest leaf of that same task, create file source-evidence-honesty-invariant.lino containing
source_evidence_honesty
  record_type meta_invariant
  observation "emit source evidence only after exact bytes are captured"
  replay "preserve URL fetched_at SHA-256 and bytes"
  failure "treat transport and cache misses as diagnostics never evidence"
  learning "derive a human-gated proposal from captured observations"'

TASK="$TASK" \
EXPECT_FILE="source-evidence-honesty-invariant.lino" \
EXPECT_TEXT="diagnostics never evidence" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8844}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/source-evidence-honesty-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/source-evidence-honesty-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
