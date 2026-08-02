#!/usr/bin/env bash
# Reproduce the Agent-CLI-authored memory-program catalog byte for byte.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-708/self-hosting-seed"
CANONICAL="$ROOT/data/seed/memory-programs.lino"
SEED_CONTENT="$(<"$CANONICAL")"
TASK="Create a Links Notation file named memory-programs.lino with exactly this content:
$SEED_CONTENT"

TASK="$TASK" \
EXPECT_FILE="memory-programs.lino" \
EXPECT_TEXT="bounded_iterate_to_fixpoint" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8709}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/memory-programs.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/memory-programs.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
