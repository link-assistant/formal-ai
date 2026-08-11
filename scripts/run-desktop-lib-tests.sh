#!/usr/bin/env bash
# Run the dependency-free desktop library tests under desktop/scripts/.
#
# Issue #812: this was `node --test $(ls ... | grep -v ...)`. If the glob ever
# stopped matching, the substitution collapsed to nothing and `node --test` fell
# back to its own directory discovery -- a green step that ran none of these
# tests. Build the list explicitly and refuse to pass on an empty one.
#
# `web-tools.test.mjs` and `command-runner.test.mjs` are excluded on purpose:
# they import production packages through desktop/lib, which only exist after
# dependency installation in desktop/. The command-runner test is run by the
# cross-platform Desktop Release matrix after that installation.
#
# Issue #977: extracted from release.yml to keep that file under the 2000-line
# ceiling scripts/check-file-size.rs enforces, and so the empty-glob guard can
# be exercised directly.
set -euo pipefail

shopt -s nullglob
tests=()
for test_file in desktop/scripts/*.test.mjs; do
  case "$test_file" in
    */web-tools.test.mjs|*/command-runner.test.mjs) continue ;;
  esac
  tests+=("$test_file")
done

if [ ${#tests[@]} -eq 0 ]; then
  echo "::error::No desktop library tests matched desktop/scripts/*.test.mjs" >&2
  exit 1
fi

printf 'Running %s test file(s)\n' "${#tests[@]}"
node --test "${tests[@]}"
