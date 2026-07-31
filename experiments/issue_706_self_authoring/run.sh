#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #706 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-706/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/language-protocol-invariant.lino"
TASK='Implement Formal AI issue #706 as five reviewable leaves: a seed-owned language registry, a generated N×N round-trip matrix, a partial fifth-language proof, an explicit language-gap policy, and a code-free sixth-language dry run. As one smallest leaf of that same task, create file language-protocol-invariant.lino containing
language_protocol_contract
  record_type meta_invariant
  registry "derive language coordination from one seed ledger"
  partial "answer only covered meanings and emit language_gap otherwise"
  scaling "generate same-language and every directed language pair from the ledger"'

TASK="$TASK" \
EXPECT_FILE="language-protocol-invariant.lino" \
EXPECT_TEXT="emit language_gap otherwise" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8706}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/language-protocol-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/language-protocol-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
