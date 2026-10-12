#!/usr/bin/env bash
# Run the test executables `collect-build-artifacts.sh` gathered.
#
# Issue #1055: these were compiled once by the shared build job, so this only
# runs them. TEST_SUITE picks the suite (PR #1188, R1188-U9):
#
# - `full` (the default): every target, skipping the suites that run apart --
#   `data_files::` and `self_ast_census` as separate steps, `specification::`
#   as its own lane -- since running them here again would be the duplication
#   issue #1037 removed;
# - `specification`: only the `specification::` tests of the unit target,
#   which the specification lane used to compile again in every shard.
set -euo pipefail

# Issue #1138: the branch suites shell out to `rust-script` for their gate
# probes (tests/unit/docs_requirements/*, scripts gated with `--test`), and the
# Test job stopped installing it when compiling moved out -- the installer is
# idempotent, so a runner or a local caller that already has it skips this.
bash scripts/install-rust-script.sh

# Issue #1138, plan 10 leaf 10: the corpus gate answers all ~1355 benchmark
# prompts through the full engine (842-873s locally, >1154s unfinished in CI
# run 35893508679), which cannot share this step's budget with the unit
# phase. It has its own workflow, benchmark-corpus-gate.yml -- the same
# partition data_files, self_ast_census, and specification use above.
# An array, not a quoted string: "$CORPUS_GATE_SKIP" would hand libtest one
# argument containing a space, which it rejects as
# `Unrecognized option: 'skip issue_1138_no_silent_unknown'` (run 35912188200,
# job 107358064235 -- the unit binary refused to start and the lane exited 101
# twenty-six seconds in). "${CORPUS_GATE_SKIP[@]}" expands to the two words.
CORPUS_GATE_SKIP=(--skip issue_1138_no_silent_unknown)

# Sharding (PR #1188): the full lane runs as SHARD_TOTAL parallel jobs. Every
# job lists all three targets and hands the one listing to
# scripts/plan-test-shards.mjs, which splits it longest-first from
# data/meta/test-durations.lino -- the same plan in every job, each test in
# exactly one shard -- and runs only its own part. Dealing tests out by listed
# index instead left one shard with the slow tests while the others idled.
# `--check` re-proves on the real listing that the shards cover every test
# exactly once. SHARD_RESERVED_SECONDS (`<shard>=<seconds>,...`) is work a
# shard already carries in other steps, so the plan gives it fewer tests.
# Every target runs even after one fails, so a single run reports every
# failure instead of stopping at the first red target.
#
# Existing per-test durations drive the stable shard plan. Suite wall timing
# comes from the CI step timestamps without enabling unstable test features.
SHARD_INDEX="${SHARD_INDEX:-1}"
SHARD_TOTAL="${SHARD_TOTAL:-1}"
SHARD_RESERVED_SECONDS="${SHARD_RESERVED_SECONDS:-}"
TEST_SUITE="${TEST_SUITE:-full}"
case "$TEST_SUITE" in
  full)
    targets=(unit integration source)
    selection=(--skip data_files:: --skip self_ast_census --skip specification:: "${CORPUS_GATE_SKIP[@]}")
    ;;
  specification)
    targets=(unit)
    selection=(specification::)
    ;;
  *)
    echo "::error::unknown TEST_SUITE '$TEST_SUITE' (full or specification)" >&2
    exit 2
    ;;
esac
status=0
if [ "$SHARD_TOTAL" -gt 1 ]; then
  listing="$(mktemp)"
  plan="$(mktemp)"
  trap 'rm -f "$listing" "$plan"' EXIT
  for target in "${targets[@]}"; do
    "dist/tests/$target" --list --format terse "${selection[@]}" \
      | sed -n 's/: test$//p' \
      | awk -v target="$target" '{ print target "\t" $0 }' >> "$listing" || status=1
  done
  node scripts/plan-test-shards.mjs --of "$SHARD_TOTAL" --reserve "$SHARD_RESERVED_SECONDS" --check < "$listing"
  node scripts/plan-test-shards.mjs --of "$SHARD_TOTAL" --reserve "$SHARD_RESERVED_SECONDS" --report < "$listing"
  node scripts/plan-test-shards.mjs --shard "$SHARD_INDEX" --of "$SHARD_TOTAL" --reserve "$SHARD_RESERVED_SECONDS" \
    < "$listing" > "$plan"
fi
for target in "${targets[@]}"; do
  if [ "$SHARD_TOTAL" -gt 1 ]; then
    mapfile -t names < <(awk -F '\t' -v target="$target" '$1 == target { print $2 }' "$plan")
    if [ "${#names[@]}" -eq 0 ]; then
      continue
    fi
    echo "shard ${SHARD_INDEX}/${SHARD_TOTAL}: ${#names[@]} ${target} test(s)"
    "dist/tests/$target" --exact "${names[@]}" || status=1
  else
    "dist/tests/$target" "${selection[@]}" || status=1
  fi
done
exit "$status"
