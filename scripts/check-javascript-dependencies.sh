#!/usr/bin/env bash
# Audit every committed JavaScript lockfile from one fail-closed CI gate.
#
# Fail-closed means an advisory fails the gate. It does not mean an unanswered
# registry does. In run 100928011479 `Lint and Format Check` spent five minutes
# inside `bun audit` and exited with
#
#   error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503
#
# npmjs.org had said nothing at all about `bun.lock`, and the branch was marked
# red for it. So the two outcomes are told apart: a registry that answers with a
# 5xx or does not answer is retried, and only a registry that answered -- with
# advisories, or with anything this script does not recognise as a transport
# fault -- ends the gate. The recognition is deliberately narrow, because an
# unmatched failure falls through to the failing branch, which is the safe one.
#
# Environment:
#   FORMAL_AI_AUDIT_ATTEMPTS             attempts before giving up (3)
#   FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS  pause between attempts (5)

set -euo pipefail

attempts="${FORMAL_AI_AUDIT_ATTEMPTS:-3}"
retry_delay_seconds="${FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS:-5}"

# What "the registry never answered" looks like from `bun audit` (`- 503`) and
# from `npm audit` (its `network`/`code E*` transport errors). Anything else --
# including every advisory report -- is an answer.
unreachable_registry='- 5[0-9][0-9]$|ECONNRESET|ECONNREFUSED|ETIMEDOUT|ENOTFOUND|EAI_AGAIN|socket hang up|network request to'

audit_with_retry() {
  local lock="$1"
  shift
  local attempt status output

  for ((attempt = 1; attempt <= attempts; attempt++)); do
    status=0
    output="$("$@" 2>&1)" || status=$?
    printf '%s\n' "$output"
    [ "$status" -eq 0 ] && return 0

    # `--` because the pattern opens with `- 5xx`, which grep would
    # otherwise read as options and refuse -- turning every audit into a
    # usage error rather than a verdict.
    if ! grep -Eq -- "$unreachable_registry" <<<"$output"; then
      return "$status"
    fi

    echo "::warning title=advisory registry unreachable for ${lock}::\
Attempt ${attempt}/${attempts} exited ${status} without an answer about this \
lockfile. Retrying; a registry outage is not a finding." >&2

    [ "$attempt" -lt "$attempts" ] || break
    sleep "$retry_delay_seconds"
  done

  echo "::error title=advisory registry unreachable for ${lock}::\
All ${attempts} attempts failed to reach the advisory registry, so this \
lockfile is unaudited. This gate stays closed rather than passing a lockfile \
nobody checked -- re-run once the registry answers." >&2
  return "$status"
}

audit_locks() {
  local lock_name="$1"
  shift
  local -a audit_command=("$@")
  local lock
  local workspace

  while IFS= read -r lock; do
    # A deleted-but-unstaged file can remain in the local index. CI never has
    # that state, but skipping it keeps this check useful while preparing a
    # commit that removes or renames a lock.
    [[ -f "$lock" ]] || continue
    workspace="$(dirname "$lock")"
    echo "Auditing $lock"
    (
      cd "$workspace"
      audit_with_retry "$lock" "${audit_command[@]}"
    )
  done < <(git ls-files | awk -F/ -v name="$lock_name" '$NF == name')
}

audit_locks "bun.lock" bun audit --audit-level=moderate
audit_locks "package-lock.json" npm audit --package-lock-only --audit-level=moderate
