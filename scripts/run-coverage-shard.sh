#!/usr/bin/env bash
# Run one shard of the instrumented test suite that coverage.yml measures.
#
# PR #1188: `Coverage / Code Coverage` ran the whole instrumented suite in one
# job -- 59-83 minutes, 72 of them in the `unit` target alone (run
# 37652295937: 4683 tests in 4310s). `coverage-build` now compiles the
# instrumented executables once; each `coverage-shard` leg runs this script
# over them, and the verdict job merges every shard's profile into the one
# report the ratchet reads.
#
# A test belongs to exactly one shard: every executable's `--list` goes into
# one listing of `<target>\t<test>` lines, and scripts/plan-test-shards.mjs
# splits that listing longest-first from data/meta/test-durations.lino (PR
# #1188) -- the same plan in every shard, which `--check` re-proves covers
# every listed test exactly once. Dealing tests out by running position left
# the shards between 11 and 18 minutes (run 37753750834) while the merge
# waited on the slowest. Every executable runs even after one fails, so a
# single shard reports every failure it owns.
#
# The executables run from the crate root, as `cargo test` runs them, with
# LLVM_PROFILE_FILE inherited from the caller so every process -- the test
# harness and each `formal-ai` it spawns -- writes its own raw profile.
#
# Usage: run-coverage-shard.sh <manifest> [libtest filter args...]
#   manifest   lines of `<target name>\t<executable path relative to the
#              repository root>`, written by coverage-build
#   SHARD_INDEX  1-based shard number
#   SHARD_TOTAL  number of shards
set -euo pipefail

manifest="${1:?a manifest of test executables is required}"
shift
: "${SHARD_INDEX:?SHARD_INDEX is required}"
: "${SHARD_TOTAL:?SHARD_TOTAL is required}"
: "${LLVM_PROFILE_FILE:?LLVM_PROFILE_FILE is required, or the run measures nothing}"
if ! [ "$SHARD_INDEX" -ge 1 ] 2>/dev/null || [ "$SHARD_INDEX" -gt "$SHARD_TOTAL" ]; then
  echo "::error::SHARD_INDEX must be within 1..SHARD_TOTAL, got $SHARD_INDEX/$SHARD_TOTAL" >&2
  exit 2
fi

root="$(pwd)"
# Listing runs each instrumented harness too. Its profile goes to a scratch
# directory, so the report counts what the tests executed and nothing that
# only enumerating them did.
list_profiles="$(mktemp -d)"
trap 'rm -rf "$list_profiles"' EXIT
[ -s "$manifest" ] || {
  echo "::error::$manifest lists no test executables; the shard would measure nothing" >&2
  exit 1
}

listing="$list_profiles/tests.tsv"
plan="$list_profiles/plan.tsv"
status=0
while IFS=$'\t' read -r name path; do
  [ -n "$name" ] || continue
  executable="$root/$path"
  [ -x "$executable" ] || {
    echo "::error::$name: $path is not an executable in this checkout" >&2
    status=1
    continue
  }
  (cd rust && LLVM_PROFILE_FILE="$list_profiles/list-%p-%m.profraw" \
    "$executable" --list --format terse "$@") \
    | sed -n 's/: test$//p' \
    | awk -v name="$name" '{ print name "\t" $0 }' >> "$listing" || status=1
done < "$manifest"
touch "$listing"
listed_total="$(wc -l < "$listing" | tr -d ' ')"
if [ "$listed_total" -eq 0 ]; then
  echo "::error::no executable listed a single test; the shard measured nothing" >&2
  exit 1
fi
node scripts/plan-test-shards.mjs --of "$SHARD_TOTAL" --check < "$listing"
node scripts/plan-test-shards.mjs --shard "$SHARD_INDEX" --of "$SHARD_TOTAL" < "$listing" > "$plan"

selected_total=0
while IFS=$'\t' read -r name path; do
  [ -n "$name" ] || continue
  executable="$root/$path"
  [ -x "$executable" ] || continue
  mapfile -t names < <(awk -F '\t' -v name="$name" '$1 == name { print $2 }' "$plan")
  if [ "${#names[@]}" -eq 0 ]; then
    continue
  fi
  of_target="$(awk -F '\t' -v name="$name" '$1 == name' "$listing" | wc -l | tr -d ' ')"
  selected_total=$((selected_total + ${#names[@]}))
  echo "coverage shard ${SHARD_INDEX}/${SHARD_TOTAL}: ${#names[@]} of ${of_target} ${name} test(s)"
  (cd rust && "$executable" --exact "${names[@]}") || status=1
done < "$manifest"

echo "coverage shard ${SHARD_INDEX}/${SHARD_TOTAL}: ran ${selected_total} of ${listed_total} test(s)"
exit "$status"
