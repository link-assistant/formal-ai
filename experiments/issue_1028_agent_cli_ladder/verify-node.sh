#!/usr/bin/env bash
# External verifier for one Agent CLI ladder node (issues #1028, #1066, #1085).
#
# The verdict never trusts the agent's own words: a leaf must have changed the
# one tracked file its task names, the file must still format, compile and pass
# the unit tests of its module; a depth-4 composite must compose both verified
# child effects and both children's diffs must apply to one tree and compile;
# a node at depth 3 or above is requirement-shaped, so every leaf marker under
# it must be present, only those files modified, and the tree must compile and
# pass the tests of every touched module (issue #1085 D4).
#
# Environment (all optional):
#   LADDER_CARGO_CHECK=0       skip `cargo check --lib`
#   LADDER_CARGO_TEST=0        skip `cargo test --test unit <module>`
#   LADDER_CARGO_TARGET_DIR    shared target directory (run.sh sets one per run)
#   LADDER_LEAVES              leaves.tsv (requirement-shaped nodes)
#   LADDER_LEAF_SPAN=start-end leaf numbers under a requirement-shaped node
#   LADDER_CHILD_DIFFS         two diff files a depth-4 composite must merge
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
report="$workspace/.agent-ladder/verify.tsv"
mkdir -p "$workspace/.agent-ladder"
: > "$report"

fail() {
  printf '%s\n' "$1"
  exit 1
}
record() {
  printf '%s\t%s\n' "$1" "$2" >> "$report"
}
target_dir() {
  printf '%s' "${LADDER_CARGO_TARGET_DIR:-$workspace/target}"
}
# The unit-test filter for a module path: its file stem, or the directory name
# for a `mod.rs`.
module_filter() {
  local stem
  stem=$(basename "$1" .rs)
  if [[ "$stem" == mod ]]; then
    stem=$(basename "$(dirname "$1")")
  fi
  printf '%s' "$stem"
}
cargo_check() {
  [[ "${LADDER_CARGO_CHECK:-1}" != 0 ]] || { record compile skipped; return 0; }
  command -v cargo >/dev/null 2>&1 || { record compile unavailable; return 0; }
  if (cd "$workspace" && CARGO_TARGET_DIR="$(target_dir)" \
      cargo check --lib --quiet >"$workspace/.agent-ladder/cargo-check.log" 2>&1); then
    record compile ok
    return 0
  fi
  tail -40 "$workspace/.agent-ladder/cargo-check.log" >&2 || true
  record compile failed
  return 1
}
# Run the unit tests whose names carry the module's stem. A module with no
# tests runs zero and passes; the count is recorded so the README says so.
cargo_test() {
  local filter="$1" log passed
  [[ "${LADDER_CARGO_TEST:-1}" != 0 ]] || { record "tests:$filter" skipped; return 0; }
  command -v cargo >/dev/null 2>&1 || { record "tests:$filter" unavailable; return 0; }
  log="$workspace/.agent-ladder/cargo-test-$filter.log"
  if (cd "$workspace" && CARGO_TARGET_DIR="$(target_dir)" \
      cargo test --test unit --quiet -- "$filter" >"$log" 2>&1); then
    passed=$(sed -n 's/^test result: ok\. \([0-9]*\) passed.*/\1/p' "$log" | tail -1)
    record "tests:$filter" "passed=${passed:-0}"
    return 0
  fi
  tail -40 "$log" >&2 || true
  record "tests:$filter" failed
  return 1
}
rustfmt_parses() {
  [[ "$1" == *.rs ]] || return 0
  command -v rustfmt >/dev/null 2>&1 || return 0
  rustfmt --edition 2024 --emit stdout "$workspace/$1" >/dev/null 2>&1
}
diff_lines() {
  git -C "$workspace" diff --numstat -- "$@" | awk '{ added += $1; removed += $2 } END { printf "%d", added + removed }'
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
effect_status=$(git -C "$workspace" status --porcelain=v1 --untracked-files=all -- "$relative")
[[ "$effect_status" == "?? $relative" ]] || fail effect_not_added
grep -Fxq "node_path=$node" "$effect" || fail bad_effect_node
grep -Fxq "node_depth=$depth" "$effect" || fail bad_effect_depth
if [[ "$depth" -eq 5 ]]; then
  grep -Fxq "node_kind=leaf" "$effect" || fail bad_effect_kind
elif [[ "$depth" -eq 4 ]]; then
  grep -Fxq "node_kind=composite" "$effect" || fail bad_effect_kind
  [[ -n "$left" ]] || fail missing_left_child
  [[ -n "$right" ]] || fail missing_right_child
  grep -Fxq "left_child=$left" "$effect" || fail missing_left_child
  grep -Fxq "right_child=$right" "$effect" || fail missing_right_child
else
  grep -Fxq "node_kind=requirement" "$effect" || fail bad_effect_kind
fi
result=$(sed -n 's/^result=//p' "$effect" | sed -n '1p')
[[ -n "$result" ]] || fail missing_effect_result
[[ ! "$result" =~ ^\<[^\<\>]+\>$ ]] || fail placeholder_effect_result
result_words=$(printf '%s\n' "$result" | awk '{ print NF }')
[[ "$result_words" -ge 4 ]] || fail hollow_effect_result
printf '%s\n' "$result" | grep -Eiq '^recorded (the )?findings([[:space:]]|$)' \
  && fail status_only_effect_result

if [[ "$depth" -eq 5 ]]; then
  [[ -n "$criterion_path" ]] || fail missing_leaf_criterion
  [[ -n "$criterion_marker" ]] || fail missing_leaf_criterion
  [[ -n "$criterion_guard" ]] || fail missing_leaf_criterion
  [[ "$criterion_path" != /* && "$criterion_path" != *".."* ]] \
    || fail invalid_leaf_criterion
  git -C "$workspace" ls-files --error-unmatch -- "$criterion_path" >/dev/null 2>&1 \
    || fail untracked_leaf_criterion
  criterion_file="$workspace/$criterion_path"
  [[ -f "$criterion_file" ]] || fail invalid_leaf_criterion
  git -C "$workspace" show "HEAD:$criterion_path" | grep -Fq -- "$criterion_marker" \
    && fail preexisting_leaf_change
  grep -Fq -- "$criterion_marker" "$criterion_file" || fail missing_leaf_change
  grep -Fq -- "$criterion_guard" "$criterion_file" || fail destroyed_leaf_anchor
  tracked_changes=$(git -C "$workspace" status --porcelain=v1 --untracked-files=no)
  [[ "$tracked_changes" == " M $criterion_path" ]] || fail unexpected_tracked_changes
  record diff_lines "$(diff_lines "$criterion_path")"
  rustfmt_parses "$criterion_path" || fail unparsable_leaf_change
  if [[ "$criterion_path" == *.rs ]]; then
    cargo_check || fail uncompilable_leaf_change
    cargo_test "$(module_filter "$criterion_path")" || fail failing_leaf_tests
  fi
  [[ "$result" == *"$criterion_marker"* ]] || fail unverified_leaf_result
elif [[ "$depth" -eq 4 ]]; then
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
  # Issue #1085 (D4): both children's diffs land in one tree and it compiles.
  if [[ -n "${LADDER_CHILD_DIFFS:-}" ]]; then
    merged_paths=()
    for child_diff in $LADDER_CHILD_DIFFS; do
      [[ -s "$child_diff" ]] || fail missing_child_diff
      git -C "$workspace" apply --check "$child_diff" >/dev/null 2>&1 \
        || fail unmergeable_child_diffs
      git -C "$workspace" apply "$child_diff" >/dev/null 2>&1 \
        || fail unmergeable_child_diffs
      while IFS= read -r changed; do
        [[ -n "$changed" ]] && merged_paths+=("$changed")
      done < <(git -C "$workspace" apply --numstat "$child_diff" 2>/dev/null | cut -f3)
    done
    record diff_lines "$(diff_lines "${merged_paths[@]}")"
    for changed in "${merged_paths[@]}"; do
      rustfmt_parses "$changed" || fail unparsable_composite_change
    done
    cargo_check || fail uncompilable_composite_change
  fi
else
  # Issue #1085 (D4): depth 3 and above are requirement-shaped. The prompt named
  # behaviour, not files; every leaf marker under the node must be present,
  # nothing outside those files may change, and the tree must compile and pass
  # the tests of every touched module. Per-leaf anchors are not checked here
  # because two leaves in one tree can legitimately rewrite each other's anchor.
  [[ -n "${LADDER_LEAVES:-}" && -n "${LADDER_LEAF_SPAN:-}" ]] || fail missing_requirement_span
  span_start="${LADDER_LEAF_SPAN%-*}"
  span_end="${LADDER_LEAF_SPAN#*-}"
  expected_paths=()
  touched=()
  while IFS=$'\t' read -r leaf _text leaf_path leaf_marker _guard _requirement; do
    number=$((10#${leaf#L}))
    (( number >= span_start && number <= span_end )) || continue
    git -C "$workspace" show "HEAD:$leaf_path" | grep -Fq -- "$leaf_marker" \
      && fail "preexisting_requirement_change:$leaf"
    grep -Fq -- "$leaf_marker" "$workspace/$leaf_path" \
      || fail "missing_requirement_change:$leaf"
    expected_paths+=("$leaf_path")
  done < "$LADDER_LEAVES"
  [[ "${#expected_paths[@]}" -gt 0 ]] || fail missing_requirement_span
  while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    [[ "$line" == " M "* ]] || fail unexpected_tracked_changes
    changed="${line#\ M }"
    allowed=0
    for expected in "${expected_paths[@]}"; do
      [[ "$changed" == "$expected" ]] && allowed=1
    done
    [[ "$allowed" -eq 1 ]] || fail unexpected_tracked_changes
    touched+=("$changed")
  done < <(git -C "$workspace" status --porcelain=v1 --untracked-files=no)
  [[ "${#touched[@]}" -gt 0 ]] || fail missing_requirement_change
  record diff_lines "$(diff_lines "${touched[@]}")"
  for changed in "${touched[@]}"; do
    rustfmt_parses "$changed" || fail unparsable_requirement_change
  done
  cargo_check || fail uncompilable_requirement_change
  filters=$(for changed in "${touched[@]}"; do module_filter "$changed"; echo; done | sort -u)
  for filter in $filters; do
    cargo_test "$filter" || fail failing_requirement_tests
  done
fi
printf 'ok\n'
