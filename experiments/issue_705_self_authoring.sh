#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #705 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-705/self-hosting-authorship"
TASK='Implement one smallest declarative leaf of Formal AI issue #705: preserve the review contract for deterministic anticipatory dreaming. Create file anticipation-invariant.lino containing
anticipation_contract
  record_type meta_invariant
  prediction "count symbolic transitions over formalized append-only request classes"
  expansion "derive variants from seeded meanings operations and observed parameters"
  probe "replay every variant offline and preserve every failure on the adoption frontier"
  prelearning "require fetch consent and retain source provenance and ttl"
  safety "keep self-extension proposal-only and human-gated"
  ledger "record predictions probes sources and later hits without inflating zero percent"'

BIN="$ROOT/target/debug/formal-ai" \
TASK="$TASK" \
EXPECT_FILE="anticipation-invariant.lino" \
EXPECT_TEXT="record predictions probes sources and later hits without inflating zero percent" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8705}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
