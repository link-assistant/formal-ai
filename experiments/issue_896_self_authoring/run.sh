#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #896 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-896/self-hosting-authorship"
CANONICAL="$ROOT/data/meta/web-component-boundary-invariant.lino"
TASK='Fix Formal AI issue #896 by adopting published web-search and web-capture component APIs in native and browser production paths while preserving exact capture provenance, cache replay, cancellation, and explicit errors. As one smallest leaf of that same task, create file web-component-boundary-invariant.lino containing
web_component_boundary
  record_type meta_invariant
  search "normalize provider captures with web-capture then fuse with web-search"
  provenance "retain exact URL fetched_at SHA-256 bytes and provider ranks"
  fallback "use the bounded local adapter only when a component capability is unavailable"
  failure "record component errors as diagnostics never source evidence"'

TASK="$TASK" \
EXPECT_FILE="web-component-boundary-invariant.lino" \
EXPECT_TEXT="component errors as diagnostics never source evidence" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8846}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/web-component-boundary-invariant.lino" "$CANONICAL"
cmp "$ARTIFACT_DIR/web-component-boundary-invariant.lino" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
