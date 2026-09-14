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
# under its own deadline, and the time an unanswered lockfile spends is charged
# to a budget that stops it. Only failures are charged: a slow audit that
# returns a verdict is not the problem this gate has.
#
# The deadline is sized from a healthy run, not from a guess. `npm audit
# --package-lock-only` over `desktop/package-lock.json` -- the largest lockfile
# here -- returned `found 0 vulnerabilities` in 2m01s on one run and had still
# not answered at 300s on the next. Both halves matter: a deadline near two
# minutes kills work that was about to succeed, and no deadline at all waits on
# a request that is never coming back.
#
#   run 100973301529  every audit *succeeded* and the job was cancelled anyway.
#                     The five lockfiles took 92s, 97s, 155s and 203s-and-
#                     counting, one after another, and 15 minutes ran out with
#                     one still unaudited.
#
# Which is the third lesson, and the reason the lockfiles are audited
# concurrently rather than in sequence. Neither limit above can help there:
# both are about a registry that is not answering, and this registry answered
# every time -- just slowly, five times over. Serially the gate costs the sum of
# five waits; concurrently it costs the longest one. Nothing is relaxed to buy
# that, because the waits were never competing for anything: five independent
# lockfiles, five independent requests. Each carries its own budget, so the
# gate's whole cost is bounded by one lockfile's worst case rather than by five
# of them added up, and that bound is far short of the job's.
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
#   FORMAL_AI_AUDIT_BUDGET_SECONDS       seconds one lockfile may spend on
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

# Seconds this lockfile has spent on attempts that never answered. Each audit
# runs in its own subshell and so keeps its own count, which is what makes the
# gate's cost the longest lockfile rather than the sum. Charging only the
# failures is the other half: a healthy audit may take as long as it likes
# without bringing the gate any closer to giving up.
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

# One temporary file per stream per lockfile. Concurrent audits would otherwise
# interleave their output line by line, and a log nobody can attribute to a
# lockfile is not evidence about any of them.
audit_output_dir="$(mktemp -d)"
trap 'rm -rf "$audit_output_dir"' EXIT

audit_pids=()
audit_names=()

start_audit() {
  local lock="$1"
  shift
  local slot="${#audit_pids[@]}"

  # The subshell is deliberate here, where it was a bug when the audits ran one
  # at a time: each lockfile wants its own working directory and its own outage
  # budget, and now that they overlap, nothing may be shared between them.
  (
    cd "$(dirname "$lock")" || exit 1
    audit_with_retry "$lock" "$@"
  ) >"$audit_output_dir/$slot.out" 2>"$audit_output_dir/$slot.err" &

  audit_pids+=("$!")
  audit_names+=("$lock")
}

# Wait for every audit, then replay them in the order they were started, so the
# log reads the same whichever one happened to finish first. Every audit is
# waited for even after one has failed: the gate reports on all five lockfiles,
# not on however many were audited before the first complaint.
wait_for_audits() {
  local slot status
  local failure=0

  for slot in "${!audit_pids[@]}"; do
    status=0
    wait "${audit_pids[slot]}" || status=$?
    echo "Auditing ${audit_names[slot]}"
    cat "$audit_output_dir/$slot.out"
    cat "$audit_output_dir/$slot.err" >&2
    [ "$status" -eq 0 ] || failure="$status"
  done

  return "$failure"
}

queue_locks() {
  local lock_name="$1"
  shift
  local lock

  while IFS= read -r lock; do
    # A deleted-but-unstaged file can remain in the local index. CI never has
    # that state, but skipping it keeps this check useful while preparing a
    # commit that removes or renames a lock.
    [[ -f "$lock" ]] || continue
    start_audit "$lock" "$@"
  done < <(git ls-files | awk -F/ -v name="$lock_name" '$NF == name')
}

queue_locks "bun.lock" bun audit --audit-level=moderate
queue_locks "package-lock.json" npm audit --package-lock-only --audit-level=moderate
wait_for_audits
