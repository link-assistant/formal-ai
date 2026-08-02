#!/usr/bin/env bash
# Prove a literal file payload remains authoritative when its contents happen
# to contain an edit-shaped phrase such as "rename X to Y" (issue #708).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
PAYLOAD='prefix rename X to Y suffix'
TASK="Create file issue-708-literal-payload.txt with exactly this content:
$PAYLOAD"

TASK="$TASK" \
EXPECT_FILE="issue-708-literal-payload.txt" \
EXPECT_TEXT="$PAYLOAD" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8808}" \
ARTIFACT_DIR="$STAGING" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cmp "$STAGING/issue-708-literal-payload.txt" <(printf '%s' "$PAYLOAD")
grep -q 'formal-ai/formal-ai' "$STAGING/agent-cli.log"
