#!/usr/bin/env bash
# Real Formal AI -> Agent CLI authors the bounded language-order policy leaf.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ARTIFACT="$ROOT/docs/case-studies/issue-709/agent-cli-evidence/language-grammar"
TASK='Continue Formal AI issue #709: serialize semantic roles in the user language without hard-coding language branches in either native Rust or browser WASM. As the bounded language-order policy leaf, create search-fusion-language-grammar.lino containing
search_fusion_language_grammar
  schema_version 1
  fallback_order "subject predicate object"
  language en
    order "subject predicate object"
  language ru
    order "subject predicate object"
  language hi
    order "subject object predicate"
  language zh
    order "subject predicate object"'

TASK="$TASK" EXPECT_FILE="search-fusion-language-grammar.lino" \
  EXPECT_TEXT='order "subject object predicate"' MIN_POSTS=3 ATTEMPTS=3 \
  PORT="${PORT:-8715}" BIN="${BIN:-$ROOT/target/debug/formal-ai}" \
  ARTIFACT_DIR="$ARTIFACT" \
  "$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"

cp "$ARTIFACT/search-fusion-language-grammar.lino" \
  "$ROOT/data/seed/search-fusion-language-grammar.lino"
cmp "$ARTIFACT/search-fusion-language-grammar.lino" \
  "$ROOT/data/seed/search-fusion-language-grammar.lino"
grep -m1 -o 'ses_[A-Za-z0-9]*' "$ARTIFACT/agent-cli.log"
