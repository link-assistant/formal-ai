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

for target in unit integration source; do
  "dist/tests/$target" \
    --skip data_files:: --skip self_ast_census --skip specification::
done
