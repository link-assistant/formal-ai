#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for the issue #988 changelog leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT/docs/case-studies/issue-988/self-hosting-authorship"
CANONICAL="$ROOT/changelog.d/20260810_988_stock_rust_install.md"
TASK="Fix Formal AI issue #988 so cargo install works on a stock Rust image without system OpenSSL. As one smallest leaf of that task, create file 20260810_988_stock_rust_install.md containing exactly:
### Fixed
- Restored \`cargo install formal-ai --locked\` on stock Rust images by selecting web-capture's transport-independent search feature, removing transitive system OpenSSL build requirements."

TASK="$TASK" \
EXPECT_FILE="20260810_988_stock_rust_install.md" \
EXPECT_TEXT="stock Rust images" \
MIN_POSTS=3 \
ATTEMPTS=3 \
PORT="${PORT:-8858}" \
ARTIFACT_DIR="$ARTIFACT_DIR" \
"$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT_DIR/20260810_988_stock_rust_install.md" "$CANONICAL"
cmp "$ARTIFACT_DIR/20260810_988_stock_rust_install.md" "$CANONICAL"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT_DIR/agent-cli.log"
