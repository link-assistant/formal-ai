#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI authorship proof for one issue #961 leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EVIDENCE_ROOT="$ROOT/docs/case-studies/issue-961/self-hosting-authorship"
FRAGMENT="20260810_120000_issue_961_macos_ci_parity.md"
DECOMPOSITION="issue-961-task-decomposition.lino"
CHANGELOG_TASK='Finish Formal AI issue #961 by restoring macOS CI parity. As one smallest leaf of that same task, create file 20260810_120000_issue_961_macos_ci_parity.md containing exactly
### Fixed
- Restored macOS CI parity for desktop packaging, canonical session diagnostics, PTY integration tests, and Bash 3.2 seed synchronization.'

TASK="$CHANGELOG_TASK" \
EXPECT_FILE="$FRAGMENT" \
EXPECT_TEXT="Restored macOS CI parity" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8961}" \
ARTIFACT_DIR="$EVIDENCE_ROOT/changelog-session" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

DECOMPOSITION_TASK='Finish Formal AI issue #961 by restoring macOS CI parity. As a second smallest leaf of that same task, create file issue-961-task-decomposition.lino containing exactly
issue_961_task_decomposition
  total_smallest_leaves 7
  required_self_authored_leaves 2
  leaf macos_mktemp author human
  leaf canonical_proxy_log author human
  leaf portable_pty_launcher author human
  leaf bash_3_2_empty_array author human
  leaf macos_ci_matrix author human
  leaf changelog_fragment author formal_ai
  leaf reviewed_task_decomposition author formal_ai'

TASK="$DECOMPOSITION_TASK" \
EXPECT_FILE="$DECOMPOSITION" \
EXPECT_TEXT="required_self_authored_leaves 2" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${DECOMPOSITION_PORT:-8962}" \
ARTIFACT_DIR="$EVIDENCE_ROOT/decomposition-session" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$EVIDENCE_ROOT/changelog-session/$FRAGMENT" "$ROOT/changelog.d/$FRAGMENT"
cp "$EVIDENCE_ROOT/decomposition-session/$DECOMPOSITION" \
  "$ROOT/docs/case-studies/issue-961/$DECOMPOSITION"
cmp "$EVIDENCE_ROOT/changelog-session/$FRAGMENT" "$ROOT/changelog.d/$FRAGMENT"
cmp "$EVIDENCE_ROOT/decomposition-session/$DECOMPOSITION" \
  "$ROOT/docs/case-studies/issue-961/$DECOMPOSITION"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$EVIDENCE_ROOT/changelog-session/agent-cli.log"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$EVIDENCE_ROOT/decomposition-session/agent-cli.log"
