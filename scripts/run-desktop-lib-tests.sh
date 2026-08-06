#!/usr/bin/env bash
# Run the dependency-free desktop library tests under desktop/scripts/.
#
# Issue #812: this was `node --test $(ls ... | grep -v ...)`. If the glob ever
# stopped matching, the substitution collapsed to nothing and `node --test` fell
# back to its own directory discovery -- a green step that ran none of these
# tests. Build the list explicitly and refuse to pass on an empty one.
#
# `web-tools.test.mjs` is excluded on purpose: it imports
# @link-assistant/web-search through desktop/lib/web-tools.cjs, which only
# exists after `npm ci` in desktop/, so it belongs to a job that installs the
# desktop dependency tree rather than being silently skipped inside this one.
#
# Issue #977: extracted from release.yml to keep that file under the 2000-line
# ceiling scripts/check-file-size.rs enforces, and so the empty-glob guard can
# be exercised directly.
set -euo pipefail

shopt -s nullglob
tests=()
for test_file in desktop/scripts/*.test.mjs; do
  case "$test_file" in
    */web-tools.test.mjs) continue ;;
  esac
  tests+=("$test_file")
done

if [ ${#tests[@]} -eq 0 ]; then
  echo "::error::No desktop library tests matched desktop/scripts/*.test.mjs" >&2
  exit 1
fi

printf 'Running %s test file(s)\n' "${#tests[@]}"
node --test "${tests[@]}"
