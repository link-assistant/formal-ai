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

for target in unit integration source; do
  "dist/tests/$target" \
    --skip data_files:: --skip self_ast_census --skip specification:: "${CORPUS_GATE_SKIP[@]}"
done
