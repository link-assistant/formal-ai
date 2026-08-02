#!/usr/bin/env bash
# Real Formal AI -> Agent CLI authors the reviewed workspace-learning leaves.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RUNNER="$ROOT/experiments/agent_cli_e2e/run_agent_cli.sh"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
BASE_PORT="${PORT:-8850}"

run_leaf() {
  local name="$1"
  local expected="$2"
  local canary="$3"
  local task="$4"
  local artifact="$ROOT/docs/case-studies/issue-848/self-hosting-workspace-learning/$name"
  TASK="$task" EXPECT_FILE="$expected" EXPECT_TEXT="$canary" MIN_POSTS=3 \
    ATTEMPTS=3 PORT="$BASE_PORT" BIN="$BIN" ARTIFACT_DIR="$artifact" "$RUNNER"
  BASE_PORT=$((BASE_PORT + 1))
}

run_leaf learning-contract workspace-change-learning-contract.lino candidate_inert \
  'Continue Formal AI issue #848 by learning a reusable coding-task procedure from verified workspace effects without activating unreviewed behavior. As the bounded learning-contract leaf, create workspace-change-learning-contract.lino containing
workspace_change_learning_contract
  schema_version 1
  task_family verified_workspace_rewrite
  minimum_independent_executions 2
  candidate_inert true
  promotion_gate "held-out suite passes with zero failures"
  human_gate "named reviewer approval"
  persistence "content-addressed Links Notation ledger"
  replay "approved recipe executes an unseen equivalent workspace rewrite"'

run_leaf execution-policy workspace-change-execution-policy.lino accept_only_exact_observation \
  'Continue Formal AI issue #848 by turning successful coding traces into one bounded, evidence-bearing transformation procedure. As the bounded execution-policy leaf, create workspace-change-execution-policy.lino containing
workspace_change_execution_policy
  schema_version 1
  task_family verified_workspace_rewrite
  stage read_target
  stage compile_bounded_normal_markov
  stage execute_against_observed_bytes
  stage reject_no_match_or_step_limit
  stage write_complete_result
  stage read_back_exact_bytes
  stage accept_only_exact_observation
  candidate_effect inert_until_reviewed
  failure_effect no_partial_write'

run_leaf generalization-fixture workspace-change-learning-generalization.lino held_out \
  'Continue Formal AI issue #848 by proving that a learned repository rewrite applies to a new identifier instead of memorizing one benchmark sentence. As the bounded generalization-fixture leaf, create workspace-change-learning-generalization.lino containing
workspace_change_learning_generalization
  training web_search_constant
    task_family verified_workspace_rewrite
    transformation identifier_substitution
  training parser_limit_constant
    task_family verified_workspace_rewrite
    transformation identifier_substitution
  held_out cache_capacity_constant
    task_family verified_workspace_rewrite
    transformation identifier_substitution
    expected_stages 7
    expected_failures 0'

for evidence in learning-contract execution-policy generalization-fixture; do
  grep -m1 -o 'ses_[A-Za-z0-9]*' \
    "$ROOT/docs/case-studies/issue-848/self-hosting-workspace-learning/$evidence/agent-cli.log"
done
