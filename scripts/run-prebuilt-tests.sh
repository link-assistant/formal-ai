#!/usr/bin/env bash
# Run the test executables `collect-build-artifacts.sh` gathered.
#
# Issue #1055: these were compiled once by the shared build job, so this only
# runs them. The skips match what the compile-and-run form selected: those
# suites have their own jobs (`data_files::` and `self_ast_census` run as
# separate steps, `specification::` on macOS), and running them here again
# would be the duplication issue #1037 removed.
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
# RECORD_TEST_TIMES=true prints each test's duration (libtest's unstable
# `--report-time`, admitted on a stable toolchain by RUSTC_BOOTSTRAP) so
# `experiments/formal_ai_subagent/ci-durations.mjs --tests` can refresh the
# recorded durations. Off by default: the variable reaches every process the
# tests spawn.
SHARD_INDEX="${SHARD_INDEX:-1}"
SHARD_TOTAL="${SHARD_TOTAL:-1}"
SHARD_RESERVED_SECONDS="${SHARD_RESERVED_SECONDS:-}"
targets=(unit integration source)
skips=(--skip data_files:: --skip self_ast_census --skip specification:: "${CORPUS_GATE_SKIP[@]}")
time_flags=()
if [ "${RECORD_TEST_TIMES:-false}" = "true" ] \
  && RUSTC_BOOTSTRAP=1 dist/tests/unit -Z unstable-options --report-time --list >/dev/null 2>&1; then
  export RUSTC_BOOTSTRAP=1
  time_flags=(-Z unstable-options --report-time)
fi
status=0
if [ "$SHARD_TOTAL" -gt 1 ]; then
  listing="$(mktemp)"
  plan="$(mktemp)"
  trap 'rm -f "$listing" "$plan"' EXIT
  for target in "${targets[@]}"; do
    "dist/tests/$target" --list --format terse "${skips[@]}" \
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
    "dist/tests/$target" "${time_flags[@]}" --exact "${names[@]}" || status=1
  else
    "dist/tests/$target" "${time_flags[@]}" "${skips[@]}" || status=1
  fi
done
exit "$status"
