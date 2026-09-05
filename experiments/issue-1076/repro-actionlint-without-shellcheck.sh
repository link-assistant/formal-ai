#!/usr/bin/env bash
#
# Issue #1076, defect D10b: a bare `actionlint` binary does not lint `run:`
# blocks by itself. It shells out to ShellCheck for them, and when ShellCheck
# cannot be found it skips those checks, says nothing about it, and exits 0.
#
# The fixture `tests/fixtures/actionlint/shellcheck-canary.yml` is a valid
# workflow whose only defect is an unterminated double quote inside a `run:`
# block, so the exit status below is entirely a statement about whether the
# shell half of the linter ran.
#
# Usage (from the repository root):
#   bash experiments/issue-1076/repro-actionlint-without-shellcheck.sh
#
# Needs `actionlint` and `shellcheck` on PATH, or their paths in ACTIONLINT_BIN
# and SHELLCHECK_BIN. Prints a two-row table and exits 0 when the defect
# reproduces (0 without ShellCheck, non-zero with it).
set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
canary="${repo_root}/tests/fixtures/actionlint/shellcheck-canary.yml"
actionlint_bin="${ACTIONLINT_BIN:-$(command -v actionlint || true)}"
shellcheck_bin="${SHELLCHECK_BIN:-$(command -v shellcheck || true)}"

if [ ! -x "${actionlint_bin:-}" ]; then
  echo "actionlint not found. Install it, or set ACTIONLINT_BIN." >&2
  echo "  curl -fsSL https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz | tar xz -C /tmp actionlint" >&2
  exit 2
fi
if [ ! -x "${shellcheck_bin:-}" ]; then
  echo "shellcheck not found. Install it, or set SHELLCHECK_BIN." >&2
  exit 2
fi
if [ ! -f "$canary" ]; then
  echo "missing fixture: $canary" >&2
  exit 2
fi

run_case() {
  # $1 label, $2... extra actionlint flags
  local label="$1"
  shift
  local out
  out="$("$actionlint_bin" -no-color "$@" "$canary" 2>&1)"
  local status=$?
  printf '%-28s exit=%-3s findings=%s\n' \
    "$label" "$status" "$(printf '%s\n' "$out" | grep -c 'shellcheck reported issue')"
  return "$status"
}

echo "actionlint: $("$actionlint_bin" --version | head -1)"
echo "shellcheck: $("$shellcheck_bin" --version | sed -n 's/^version: //p')"
echo

# `-shellcheck` pointing at a path that does not exist is the same situation as
# an empty PATH, and is the reproducible way to stage it without uninstalling
# anything. `-shellcheck=` (empty) disables the integration outright; both were
# measured for the issue and both exit 0.
run_case "without shellcheck" -shellcheck /nonexistent/shellcheck
missing_status=$?
run_case "with shellcheck" -shellcheck "$shellcheck_bin"
present_status=$?

echo
if [ "$missing_status" -eq 0 ] && [ "$present_status" -ne 0 ]; then
  echo "REPRODUCED: the same workflow passes when ShellCheck is unreachable and fails when it is not."
  echo "Fix: run the linter as the container image, which bundles ShellCheck --"
  echo "  docker run --rm -v \"\$PWD:/repo\" -w /repo rhysd/actionlint:1.7.12 -color"
  echo "and keep a canary step that fails when the fixture passes, so the gate cannot re-narrow silently."
  exit 0
fi
echo "NOT REPRODUCED: missing=$missing_status present=$present_status (upstream behaviour may have changed)."
exit 1
