#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #709 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-709/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/search-fusion-provenance-invariant.lino"
TASK='Finish Formal AI issue #709 by formalizing every captured search statement, merging equivalent meanings across languages, ranking by source tiers, preserving both conflict sides, and rendering normalized source provenance. As one smallest leaf of that same task, create file search-fusion-provenance-invariant.lino containing
search_fusion_provenance
  record_type meta_invariant
  formalization "derive answer statements only from captured source observations"
  merge "join complete meaning links while retaining every source receipt"
  conflict "show both ranked sides with tiers and posteriors"
  presentation "render title url quote and read more in the query language"'

TASK="$TASK" \
EXPECT_FILE="search-fusion-provenance-invariant.lino" \
EXPECT_TEXT="show both ranked sides" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8709}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/search-fusion-provenance-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/search-fusion-provenance-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
