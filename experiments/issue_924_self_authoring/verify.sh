#!/usr/bin/env bash

set -euo pipefail

execution='self_development_execution_contract
  record_type "self_development_execution_contract"
  issue "924"
  task_execution "formal_ai_via_agent_cli"
  strategy "attempt_whole_then_split_only_after_failure"
  recursion "split_until_solvable_or_bounded_irreducible"
  effect_application "verified_passing_session_only"
  learning "same_sessions_to_proposal_only_learning"
  promotion "human_review_required"'
pull_request='self_development_pull_request_contract
  record_type "self_development_pull_request_contract"
  issue "924"
  authorship "end_to_end"
  commit_coverage "every_non_merge_commit_introduced_by_pull_request"
  evidence "session_and_committed_replay_per_commit"
  review_ci_promotion "unchanged"'
coordination='coordinate issue 924 self-development loop'

verify_exact() {
  local path="$1"
  local expected="$2"
  [ -f "$path" ]
  [ "$(cat "$path")" = "$expected" ]
}

goal="${FORMAL_AI_VERIFICATION_TASK:-}"
if [ -z "$goal" ]; then
  plan='.formal-ai/general-change-plan.lino'
  [ -f "$plan" ]
  goal="$(grep '^  goal ' "$plan" | tail -1)"
fi

case "$goal" in
  *self-development-execution-contract.lino*self-development-pull-request-contract.lino*)
    verify_exact issue-924-coordination.txt "$coordination"
    verify_exact self-development-execution-contract.lino "$execution"
    verify_exact self-development-pull-request-contract.lino "$pull_request"
    ;;
  *issue-924-coordination.txt*)
    verify_exact issue-924-coordination.txt "$coordination"
    ;;
  *self-development-execution-contract.lino*)
    verify_exact self-development-execution-contract.lino "$execution"
    ;;
  *self-development-pull-request-contract.lino*)
    verify_exact self-development-pull-request-contract.lino "$pull_request"
    ;;
  *)
    echo "verification could not identify the current task: $goal" >&2
    exit 1
    ;;
esac
