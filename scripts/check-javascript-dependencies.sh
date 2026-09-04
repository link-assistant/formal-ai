#!/usr/bin/env bash
# Audit every committed JavaScript lockfile from one fail-closed CI gate.
#
# Fail-closed means an advisory fails the gate. It does not mean an unanswered
# registry does, and it does not mean the gate may spend the job's whole budget
# finding that out. Two runs taught both halves:
#
#   run 100928011479  `bun audit` exited with
#                     `error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503`
#                     and the branch went red for something npmjs.org had never
#                     said about `bun.lock`.
#   run 100948708530  the retry reached a clean answer on attempt 2 in 4.35s --
#                     but attempt 1 had hung for five minutes first, and the
#                     15-minute job was cancelled two lockfiles later.
#
# So a retry is only an improvement while it can finish, the rule
# `apt-install-with-retry.sh` already carries (issue #1021). Every attempt runs
# under its own deadline, and the time spent on attempts that did not answer is
# charged to one budget shared by every lockfile. Only failures are charged: a
# slow audit that returns a verdict is not the problem, so five slow-but-healthy
# lockfiles can never exhaust a budget meant for an outage.
#
# The deadline is sized from a healthy run, not from a guess. `npm audit
# --package-lock-only` over `desktop/package-lock.json` -- the largest lockfile
# here -- returned `found 0 vulnerabilities` in 2m01s on one run and had still
# not answered at 300s on the next. Both halves matter: a deadline near two
# minutes kills work that was about to succeed, and no deadline at all waits on
# a request that is never coming back. 180s leaves the healthy measurement room
# to drift; the budget stops the gate after two such non-answers. The worst case is therefore about six minutes, comfortably inside
# the 15-minute job so that the gate's own bound is what fires, and the job's
# `timeout-minutes` stays the backstop it is meant to be (issue #1017).
#
# The three outcomes stay distinct. A registry that answers -- with advisories,
# or with anything not recognisable as a transport fault -- ends the gate on the
# first attempt. A registry that does not answer is retried. A registry that
# never answers still fails, because retrying is not passing and an unaudited
# lockfile is what this gate exists to refuse. The recognition is deliberately
# narrow: an unmatched failure falls through to the failing branch.
#
# Environment:
#   FORMAL_AI_AUDIT_ATTEMPTS             attempts per lockfile before giving up (3)
#   FORMAL_AI_AUDIT_ATTEMPT_SECONDS      deadline for one attempt (180)
#   FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS  pause between attempts (5)
#   FORMAL_AI_AUDIT_BUDGET_SECONDS       seconds the gate may spend on
#                                        attempts that never answered (300)

set -euo pipefail

attempts="${FORMAL_AI_AUDIT_ATTEMPTS:-3}"
attempt_seconds="${FORMAL_AI_AUDIT_ATTEMPT_SECONDS:-180}"
retry_delay_seconds="${FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS:-5}"
budget_seconds="${FORMAL_AI_AUDIT_BUDGET_SECONDS:-300}"

# What "the registry never answered" looks like. The two tools word it
# differently, and the first draft of this list was written from `bun audit`
# alone, so a real npm outage -- reproduced locally against a degraded
# registry -- went unrecognised and would have failed the gate outright:
#
#   npm warn audit 503 Service Unavailable - POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - Service Unavailable
#   npm warn audit network timeout at: https://registry.npmjs.org/-/npm/v1/security/advisories/bulk
#   npm error audit endpoint returned an error
#
# `npm` prints that last line whenever the endpoint itself failed, and prints
# nothing of the sort when it has advisories to report, so it is the reliable
# half of the pair. A killed attempt says the same thing by exiting 124 with
# nothing to report.
unreachable_registry='- 5[0-9][0-9]$|audit endpoint returned an error|network timeout at|ECONNRESET|ECONNREFUSED|ETIMEDOUT|ENOTFOUND|EAI_AGAIN|socket hang up|network request to'
killed_at_deadline=124

# Seconds already spent on attempts that never answered, across every lockfile
# so far. Charging only the failures is the point: a healthy audit may take as
# long as it likes without bringing the gate closer to giving up.
outage_seconds=0

audit_with_retry() {
  local lock="$1"
  shift
  local attempt status output deadline began

  status=1
  for ((attempt = 1; attempt <= attempts; attempt++)); do
    if [ "$outage_seconds" -ge "$budget_seconds" ]; then
      echo "::error title=advisory registry unreachable for ${lock}::\
The gate's ${budget_seconds}s budget ran out with this lockfile still \
unaudited. Failing here rather than spending the job's remaining time on a \
registry that is not answering -- re-run once it does." >&2
      return "$status"
    fi
    # The deadline is not trimmed to the budget that is left. The budget
    # counts non-answers, and shortening an attempt because earlier ones failed
    # would start cutting off the answer this one is waiting for. It overshoots
    # by at most one attempt, which is what makes it a bound rather than a race.
    deadline="$attempt_seconds"

    began=$SECONDS
    status=0
    output="$(timeout "$deadline" "$@" 2>&1)" || status=$?
    printf '%s\n' "$output"
    [ "$status" -eq 0 ] && return 0
    outage_seconds=$((outage_seconds + SECONDS - began))

    # `--` because the pattern opens with `- 5xx`, which grep would otherwise
    # read as options and refuse -- turning every audit into a usage error
    # rather than a verdict.
    if [ "$status" -ne "$killed_at_deadline" ] &&
      ! grep -Eq -- "$unreachable_registry" <<<"$output"; then
      return "$status"
    fi

    echo "::warning title=advisory registry unreachable for ${lock}::\
Attempt ${attempt}/${attempts} exited ${status} within its ${deadline}s \
deadline without an answer about this lockfile. Retrying; a registry outage is \
not a finding." >&2

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
  local origin
  origin="$PWD"

  while IFS= read -r lock; do
    # A deleted-but-unstaged file can remain in the local index. CI never has
    # that state, but skipping it keeps this check useful while preparing a
    # commit that removes or renames a lock.
    [[ -f "$lock" ]] || continue
    workspace="$(dirname "$lock")"
    echo "Auditing $lock"
    # Not a subshell: the outage budget is shared across lockfiles, and a
    # subshell would hand each one a fresh copy of it. `set -e` ends the script
    # on a failed audit, so the only path that needs the directory back is the
    # passing one.
    cd "$workspace"
    audit_with_retry "$lock" "${audit_command[@]}"
    cd "$origin"
  done < <(git ls-files | awk -F/ -v name="$lock_name" '$NF == name')
}

audit_locks "bun.lock" bun audit --audit-level=moderate
audit_locks "package-lock.json" npm audit --package-lock-only --audit-level=moderate
