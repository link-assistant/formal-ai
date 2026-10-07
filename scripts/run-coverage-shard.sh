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
# A test belongs to exactly one shard: tests are numbered by their position in
# one running sequence over every executable in manifest order (each
# executable's own `--list`, so the order is the libtest order), and shard i of
# n runs the positions p with p % n == i - 1. The running position, rather than
# a per-executable restart, keeps the many one-test targets from all landing
# on the first shard. Every executable runs even after one fails, so a single
# shard reports every failure it owns.
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

position=0
selected_total=0
status=0
while IFS=$'\t' read -r name path; do
  [ -n "$name" ] || continue
  executable="$root/$path"
  [ -x "$executable" ] || {
    echo "::error::$name: $path is not an executable in this checkout" >&2
    status=1
    continue
  }
  mapfile -t listed < <(cd rust && LLVM_PROFILE_FILE="$list_profiles/list-%p-%m.profraw" \
    "$executable" --list --format terse "$@" | sed -n 's/: test$//p')
  names=()
  for test_name in "${listed[@]}"; do
    if [ $((position % SHARD_TOTAL)) -eq $((SHARD_INDEX - 1)) ]; then
      names+=("$test_name")
    fi
    position=$((position + 1))
  done
  if [ "${#names[@]}" -eq 0 ]; then
    continue
  fi
  selected_total=$((selected_total + ${#names[@]}))
  echo "coverage shard ${SHARD_INDEX}/${SHARD_TOTAL}: ${#names[@]} of ${#listed[@]} ${name} test(s)"
  (cd rust && "$executable" --exact "${names[@]}") || status=1
done < "$manifest"

echo "coverage shard ${SHARD_INDEX}/${SHARD_TOTAL}: ran ${selected_total} of ${position} test(s)"
if [ "$position" -eq 0 ]; then
  echo "::error::no executable listed a single test; the shard measured nothing" >&2
  exit 1
fi
exit "$status"
