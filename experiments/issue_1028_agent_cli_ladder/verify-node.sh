#!/usr/bin/env bash
# Independently verify one real Agent ladder node after the Agent exits.
# A proof is supporting evidence, not the effect: the node also has to add a
# task-result artifact to the Git worktree, and composite results must name both
# children they claim to compose.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

if [[ "$#" -ne 9 ]]; then
  echo "usage: verify-node.sh WORKSPACE PROOF NODE DEPTH LEFT RIGHT CHANGE_PATH CHANGE_MARKER CHANGE_GUARD" >&2
  exit 2
fi

workspace="$1"
proof="$2"
node="$3"
depth="$4"
left="$5"
right="$6"
criterion_path="$7"
criterion_marker="$8"
criterion_guard="$9"

fail() {
  printf '%s\n' "$1"
  exit 1
}

[[ "$node" =~ ^(R|[12](\.[12]){0,4})$ ]] || fail invalid_node
[[ "$depth" =~ ^[0-5]$ ]] || fail invalid_depth

proof_verdict=$(python3 \
  "$ROOT/experiments/issue_1066_ladder_offline/judge-proof.py" \
  "$proof" "$node")
proof_status=$?
[[ "$proof_status" -eq 0 ]] || fail "$proof_verdict"

relative="agent-ladder-effects/node-${node}.lino"
effect="$workspace/$relative"
[[ -s "$effect" ]] || fail missing_effect

# The fixture is committed before the Agent starts. Requiring Git's untracked
# marker proves that this run introduced the effect rather than merely finding
# an artifact already present in the repository.
effect_status=$(git -C "$workspace" status --porcelain=v1 --untracked-files=all -- "$relative")
[[ "$effect_status" == "?? $relative" ]] || fail effect_not_added

grep -Fxq "node_path=$node" "$effect" || fail bad_effect_node
grep -Fxq "node_depth=$depth" "$effect" || fail bad_effect_depth

if [[ "$depth" -eq 5 ]]; then
  grep -Fxq "node_kind=leaf" "$effect" || fail bad_effect_kind
else
  grep -Fxq "node_kind=composite" "$effect" || fail bad_effect_kind
  [[ -n "$left" ]] || fail missing_left_child
  [[ -n "$right" ]] || fail missing_right_child
  grep -Fxq "left_child=$left" "$effect" || fail missing_left_child
  grep -Fxq "right_child=$right" "$effect" || fail missing_right_child
fi

result=$(sed -n 's/^result=//p' "$effect" | sed -n '1p')
[[ -n "$result" ]] || fail missing_effect_result
# An angle-bracketed value by itself is an unfilled contract placeholder. Code
# excerpts may legitimately contain generics such as `Vec<Self>` or
# `sum::<usize>()`, so angle brackets embedded in a substantive result are not
# rejected.
[[ ! "$result" =~ ^\<[^\<\>]+\>$ ]] || fail placeholder_effect_result
result_words=$(printf '%s\n' "$result" | awk '{ print NF }')
[[ "$result_words" -ge 4 ]] || fail hollow_effect_result
printf '%s\n' "$result" | grep -Eiq '^recorded (the )?findings([[:space:]]|$)' \
  && fail status_only_effect_result

if [[ "$depth" -eq 5 ]]; then
  # A leaf is change-shaped: its deliverable is a diff to a tracked source, not
  # a side file describing one. Every clause below asks the Git worktree what
  # happened rather than asking the node what it claims.
  [[ -n "$criterion_path" ]] || fail missing_leaf_criterion
  [[ -n "$criterion_marker" ]] || fail missing_leaf_criterion
  [[ -n "$criterion_guard" ]] || fail missing_leaf_criterion
  [[ "$criterion_path" != /* && "$criterion_path" != *".."* ]] \
    || fail invalid_leaf_criterion
  git -C "$workspace" ls-files --error-unmatch -- "$criterion_path" >/dev/null 2>&1 \
    || fail untracked_leaf_criterion
  criterion_file="$workspace/$criterion_path"
  [[ -f "$criterion_file" ]] || fail invalid_leaf_criterion

  # The marker has to be absent from the committed fixture, so finding it in the
  # worktree proves this node introduced it instead of discovering it.
  git -C "$workspace" show "HEAD:$criterion_path" | grep -Fq -- "$criterion_marker" \
    && fail preexisting_leaf_change
  grep -Fq -- "$criterion_marker" "$criterion_file" || fail missing_leaf_change
  # Deleting the surrounding item would also make the marker "appear" once the
  # node rewrote the file from scratch, so the anchor has to survive the edit.
  grep -Fq -- "$criterion_guard" "$criterion_file" || fail destroyed_leaf_anchor

  # Exactly one tracked file may differ, and it must be the target. Untracked
  # evidence (the effect file, the proof) is deliberately not counted.
  tracked_changes=$(git -C "$workspace" status --porcelain=v1 --untracked-files=no)
  [[ "$tracked_changes" == " M $criterion_path" ]] || fail unexpected_tracked_changes

  # A change that no longer parses is not a change a reviewer could take.
  if [[ "$criterion_path" == *.rs ]] && command -v rustfmt >/dev/null 2>&1; then
    rustfmt --edition 2024 --emit stdout "$criterion_file" >/dev/null 2>&1 \
      || fail unparsable_leaf_change
  fi

  [[ "$result" == *"$criterion_marker"* ]] || fail unverified_leaf_result
else
  child_directory=".agent-ladder/verified-children"
  left_relative="$child_directory/node-$left.lino"
  right_relative="$child_directory/node-$right.lino"
  left_effect="$workspace/$left_relative"
  right_effect="$workspace/$right_relative"
  [[ -s "$left_effect" && -s "$right_effect" ]] || fail missing_child_effect
  git -C "$workspace" ls-files --error-unmatch -- "$left_relative" "$right_relative" \
    >/dev/null 2>&1 || fail missing_child_effect
  child_status=$(git -C "$workspace" status --porcelain=v1 --untracked-files=all -- \
    "$left_relative" "$right_relative")
  [[ -z "$child_status" ]] || fail modified_child_effect
  grep -Fxq "node_path=$left" "$left_effect" || fail invalid_child_effect
  grep -Fxq "node_path=$right" "$right_effect" || fail invalid_child_effect
  left_result=$(sed -n 's/^result=//p' "$left_effect" | sed -n '1p')
  right_result=$(sed -n 's/^result=//p' "$right_effect" | sed -n '1p')
  [[ -n "$left_result" && -n "$right_result" ]] || fail invalid_child_effect
  left_claim=$(sed -n 's/^left_result=//p' "$effect" | sed -n '1p')
  right_claim=$(sed -n 's/^right_result=//p' "$effect" | sed -n '1p')
  [[ "$left_claim" == "$left_result" ]] || fail unverified_left_child_result
  [[ "$right_claim" == "$right_result" ]] || fail unverified_right_child_result
  [[ "$result" == *"$left_result"* ]] || fail uncomposed_left_child_result
  [[ "$result" == *"$right_result"* ]] || fail uncomposed_right_child_result
fi

printf 'ok\n'
