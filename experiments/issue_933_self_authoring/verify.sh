#!/usr/bin/env bash

set -euo pipefail

contract='conversational_variation_floor_contract
  record_type "conversational_variation_floor_contract"
  issue "933"
  minimum_per_language "5"
  languages "en|ru|hi|zh"
  normalization "nfkc_lowercase_strip_punctuation_symbols_separators_whitespace"
  execution "attempt_whole_then_split_on_failure"'
learning='conversational_variation_floor_learning
  record_type "conversational_variation_floor_learning"
  issue "933"
  source "verified_agent_cli_session"
  observation "incremental_dispatch"
  promotion "proposal_only"
  human_review "required"'
coordination='coordinate issue 933 artifacts'

verify_exact() {
  local path="$1"
  local expected="$2"
  [ -f "$path" ]
  [ "$(cat "$path")" = "$expected" ]
}

plan='.formal-ai/general-change-plan.lino'
[ -f "$plan" ]
goal="$(grep '^  goal ' "$plan" | tail -1)"

case "$goal" in
  *variation-floor-contract.lino*variation-floor-learning.lino*)
    verify_exact variation-floor-contract.lino "$contract"
    verify_exact variation-floor-learning.lino "$learning"
    ;;
  *issue-933-coordination.txt*)
    verify_exact issue-933-coordination.txt "$coordination"
    ;;
  *variation-floor-contract.lino*)
    verify_exact variation-floor-contract.lino "$contract"
    ;;
  *variation-floor-learning.lino*)
    verify_exact variation-floor-learning.lino "$learning"
    ;;
  *)
    echo "verification could not identify the current task: $goal" >&2
    exit 1
    ;;
esac
