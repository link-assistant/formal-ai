#!/usr/bin/env bash
# Shellcheck every tracked shell script that ships as part of the project.
#
# `experiments/`, `dev/log/` and `docs/case-studies/` are excluded on purpose:
# those hold throwaway reproduction scripts and archived evidence, not code we
# maintain.
#
# The empty-list guard matters more than it looks: if the selector ever stops
# matching, `shellcheck` with no arguments would read stdin and exit 0 -- a
# green lint step that linted nothing. That is the same false-negative shape as
# issue #812's collapsing glob.
#
# Issue #977: extracted from release.yml to keep that file under the 2000-line
# ceiling scripts/check-file-size.rs enforces.
set -euo pipefail

mapfile -t scripts < <(
  git ls-files '*.sh' \
    | grep -v -e '^experiments/' -e '^dev/log/' -e '^docs/case-studies/'
)

if [ ${#scripts[@]} -eq 0 ]; then
  echo "::error::No shell scripts matched; the lint selector is broken" >&2
  exit 1
fi

printf 'Linting %s shell script(s)\n' "${#scripts[@]}"
shellcheck --severity=warning "${scripts[@]}"
