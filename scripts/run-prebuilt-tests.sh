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

# Sharding (PR #1188): the full lane runs as SHARD_TOTAL parallel jobs, each
# taking every SHARD_TOTAL-th test (round-robin by listed index) of every
# target, so the ~25-minute suite finishes in a fraction of the time. Every
# target runs even after one fails, so a single run reports every failure
# instead of stopping at the first red target.
SHARD_INDEX="${SHARD_INDEX:-1}"
SHARD_TOTAL="${SHARD_TOTAL:-1}"
status=0
for target in unit integration source; do
  skips=(--skip data_files:: --skip self_ast_census --skip specification:: "${CORPUS_GATE_SKIP[@]}")
  if [ "$SHARD_TOTAL" -gt 1 ]; then
    mapfile -t names < <("dist/tests/$target" --list --format terse "${skips[@]}" \
      | sed -n 's/: test$//p' \
      | awk -v index_="$SHARD_INDEX" -v total="$SHARD_TOTAL" '(NR - 1) % total == index_ - 1')
    if [ "${#names[@]}" -eq 0 ]; then
      continue
    fi
    echo "shard ${SHARD_INDEX}/${SHARD_TOTAL}: ${#names[@]} ${target} test(s)"
    "dist/tests/$target" --exact "${names[@]}" || status=1
  else
    "dist/tests/$target" "${skips[@]}" || status=1
  fi
done
exit "$status"
