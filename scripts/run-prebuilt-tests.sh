#!/usr/bin/env bash
# Run the test executables `collect-build-artifacts.sh` gathered.
#
# Issue #1055: these were compiled once by the shared build job, so this only
# runs them. The skips match what the compile-and-run form selected: those
# suites have their own jobs (`data_files::` and `self_ast_census` run as
# separate steps, `specification::` on macOS), and running them here again
# would be the duplication issue #1037 removed.
set -euo pipefail

for target in unit integration source; do
  "dist/tests/$target" \
    --skip data_files:: --skip self_ast_census --skip specification::
done
