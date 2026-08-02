#!/usr/bin/env bash
# Real Formal AI -> Agent CLI authors three bounded data leaves of issue #709.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUNNER="$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
BASE_PORT="${PORT:-8710}"

run_leaf() {
  local name="$1"
  local expected="$2"
  local canary="$3"
  local task="$4"
  local artifact="$ROOT/docs/case-studies/issue-709/agent-cli-evidence/$name"
  TASK="$task" EXPECT_FILE="$expected" EXPECT_TEXT="$canary" MIN_POSTS=3 \
    ATTEMPTS=3 PORT="$BASE_PORT" BIN="$BIN" ARTIFACT_DIR="$artifact" "$RUNNER"
  BASE_PORT=$((BASE_PORT + 1))
}

run_leaf learning-contract search-fusion-learning-contract.lino candidate_inert \
  'Continue Formal AI issue #709: learn to repeat captured-source fusion without allowing unreviewed behavior changes. As the bounded learning-contract leaf, create search-fusion-learning-contract.lino containing
search_fusion_learning_contract
  schema_version 1
  task_family captured_search_statement_fusion
  minimum_independent_executions 2
  candidate_inert true
  promotion_gate "held-out suite passes with zero failures"
  human_gate "named reviewer approval"
  persistence "content-addressed Links Notation ledger"
  replay "approved recipe executes unseen equivalent research tasks"'

run_leaf source-policy search-fusion-source-policy.lino duplicate_capture \
  'Continue Formal AI issue #709: formalize every captured statement, merge cross-language meanings, tier sources, keep conflicts, and preserve normalized provenance. As the bounded policy-data leaf, create search-fusion-source-policy.lino containing
search_fusion_source_policy
  schema_version 1
  task_family captured_search_statement_fusion
  stage capture
  stage formalize_each_statement
  stage semantic_merge
  stage source_tier_rank
  stage smallest_conflict_complete_select
  stage deformalize_preserving_quote
  stage normalized_provenance_render
  language_scope statement
  duplicate_capture unoriginal
  original_precedence highest_tier_then_retrieval_rank
  unoriginal_contribution zero'

run_leaf generalization-fixture search-fusion-learning-generalization.lino held_out \
  'Continue Formal AI issue #709: prove the learned fusion procedure generalizes beyond memorized queries. As the bounded deterministic-fixture leaf, create search-fusion-learning-generalization.lino containing
search_fusion_learning_generalization
  training apple_taxonomy
    task_family captured_search_statement_fusion
    query "apple taxonomy"
  training parser_speed
    task_family captured_search_statement_fusion
    query "parser speed"
  held_out tomato_taxonomy
    task_family captured_search_statement_fusion
    query "tomato taxonomy"
    expected_stages 7
    expected_policy "formalize merge rank preserve provenance"'

cp "$ROOT/docs/case-studies/issue-709/agent-cli-evidence/learning-contract/search-fusion-learning-contract.lino" \
  "$ROOT/data/meta/search-fusion-learning-contract.lino"
cp "$ROOT/docs/case-studies/issue-709/agent-cli-evidence/source-policy/search-fusion-source-policy.lino" \
  "$ROOT/data/meta/search-fusion-source-policy.lino"
cp "$ROOT/docs/case-studies/issue-709/agent-cli-evidence/generalization-fixture/search-fusion-learning-generalization.lino" \
  "$ROOT/data/benchmarks/search-fusion-learning-generalization.lino"

for evidence in learning-contract source-policy generalization-fixture; do
  grep -m1 -o 'ses_[A-Za-z0-9]*' \
    "$ROOT/docs/case-studies/issue-709/agent-cli-evidence/$evidence/agent-cli.log"
done
