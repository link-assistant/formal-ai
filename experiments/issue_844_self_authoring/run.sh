#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #844 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-844/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/multi-source-summary-honesty-invariant.lino"
TASK='Finish Formal AI issue #844 by gathering multiple exact source captures, deduplicating their statements into one named context, fact-checking before presentation, and producing deterministic human-gated learning proposals. As one smallest leaf of that same task, create file multi-source-summary-honesty-invariant.lino containing
multi_source_summary_honesty
  record_type meta_invariant
  capture "derive evidence only from exact captured bytes"
  merge "deduplicate statements into one named context with reversible provenance"
  verification "run disproof-first fact checking before presentation"
  learning "render deterministic human-gated proposals from captures merge and audit"'

TASK="$TASK" \
EXPECT_FILE="multi-source-summary-honesty-invariant.lino" \
EXPECT_TEXT="human-gated proposals" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8845}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/multi-source-summary-honesty-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/multi-source-summary-honesty-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
