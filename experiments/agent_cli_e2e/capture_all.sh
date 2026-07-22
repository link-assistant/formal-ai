#!/usr/bin/env bash
# Capture a real, multi-recipe Agent-CLI ↔ formal-ai E2E run into the case study
# log (docs/case-studies/issue-538/agent-cli-e2e-run.log).
#
# It drives the REAL `@link-assistant/agent` CLI over the OpenAI-compatible server
# for every issue-#538 recipe axis (tomato, potato, diagrams, self-AST), so the
# committed log is a faithful record of the loop actually working — not a mock.
# Each section is the verbatim stdout of `run_agent_cli.sh` for that recipe, which
# already ends in the hard `== E2E OK: … ==` assertion line.
#
# Usage:
#   cargo build --release --bin formal-ai   # once, so target/release/formal-ai exists
#   experiments/agent_cli_e2e/capture_all.sh > docs/case-studies/issue-538/agent-cli-e2e-run.log
#
# The tmp workdir paths and timestamps differ run-to-run (real sandboxes), so this
# log is documentation, not a byte-for-byte artifact; the byte-for-byte guarantee
# lives in the issue_538_agentic tests and reproduce-issue-538.sh.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

RUN="experiments/agent_cli_e2e/run_agent_cli.sh"
COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
FA_VERSION="$("$ROOT/target/release/formal-ai" --version 2>/dev/null || echo 'formal-ai ?')"
AGENT_VERSION="$(agent --version 2>/dev/null || echo '?')"

echo "# Real Agent CLI ↔ formal-ai E2E — live multi-recipe run"
echo "# Captured $COMMIT on branch $BRANCH"
echo "# $FA_VERSION | agent $AGENT_VERSION"
echo "# Four recipe axes driven by the REAL @link-assistant/agent CLI over the OpenAI-compatible endpoint."
echo "# Meaning search/fetch turns use the committed MCP fixture for reproducible Wikidata evidence."
echo

recipe() {
  local n="$1" total="$2" title="$3"
  shift 3
  echo "==================== RECIPE $n/$total: $title ===================="
  env "$@" "$RUN"
  echo
}

recipe 1 4 "tomato meaning (search → fetch → write → verify → final)" \
  PORT=8768 \
  EXPECT_FILE=meanings-tomato-detail.lino \
  EXPECT_TEXT=томаты \
  RESEARCH_MCP_FIXTURE=experiments/agent_cli_e2e/mock-meaning-mcp.mjs

recipe 2 4 "potato meaning (different wording, same recipe)" \
  PORT=8769 \
  TASK="Please make the potato word and meaning richer — record the singular/plural of each surface, add the missing plural form potatoes, and keep it grounded in Wikidata." \
  EXPECT_FILE=meanings-potato-detail.lino \
  EXPECT_TEXT=potatoes \
  RESEARCH_MCP_FIXTURE=experiments/agent_cli_e2e/mock-meaning-mcp.mjs

recipe 3 4 "generated diagrams (non-lexeme axis, no web step: write → verify → final)" \
  PORT=8770 \
  TASK="Generate the mermaid diagrams of our agentic recipes, split into parts, as a visual overview of how Formal AI drives its own tools." \
  EXPECT_FILE=agentic-recipes.md \
  EXPECT_TEXT=flowchart \
  MIN_POSTS=3

recipe 4 4 "self-inspection CST/AST census (self-referential, no web step)" \
  PORT=8771 \
  TASK="Store the CST/AST of our Rust meta algorithm in our data so the system can reason about itself: parse the planner module and record its abstract-syntax node census in Links Notation." \
  EXPECT_FILE=self-ast.lino \
  EXPECT_TEXT=named_node_count \
  MIN_POSTS=3

echo "==================== ALL FOUR RECIPE AXES PASSED LIVE ===================="
