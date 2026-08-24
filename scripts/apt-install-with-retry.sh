#!/usr/bin/env bash
# Install Debian packages under a per-attempt deadline, retrying the transient
# mirror stalls that make a hosted runner look like our defect.
#
# Issue #1017 gave the Xvfb install a 300s budget because an unbounded
# `apt-get update` had burned a whole 25-minute job cap. The budget did its job
# in run 32272689026: `E2E (opencode-desktop)` was terminated at exactly 300s
# and reported `failure` -- while the sibling GUI legs of the same run installed
# the same package in 52s. A deadline that only *reports* a transient stall
# still spends the whole budget on one attempt and turns a green pipeline red
# for a reason no commit in it caused.
#
# So bound the attempt, not just the step: each attempt gets its own deadline,
# a stalled one is killed while the budget still has room for another, and the
# whole wrapper is designed to finish inside the enclosing step budget. When
# TEST_BUDGET_SECONDS is set, deadlines grow geometrically across attempts so
# the first probe is cheap and the last attempt receives the largest share of
# the remaining time. With the default 300s/3-attempt shape and 5s delays this
# produces 41s / 83s / 166s rather than repeating 90s / 90s / 90s.
#
# Usage: apt-install-with-retry.sh <package> [package...]
#
# Environment:
#   FORMAL_AI_APT_ATTEMPTS          attempts before giving up (3)
#   FORMAL_AI_APT_ATTEMPT_SECONDS   fixed deadline when no step budget is set (90)
#   FORMAL_AI_APT_RETRY_DELAY_SECONDS  pause between attempts (5)
#   TEST_BUDGET_SECONDS             enclosing step budget; enables escalation
#   FORMAL_AI_APT_GET               apt-get binary (apt-get); tests point it
#                                   at a stand-in
#   FORMAL_AI_APT_PRIVILEGE         privilege escalation prefix (sudo); empty
#                                   when already root, as tests are
set -euo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
deadline="$script_directory/run-with-deadline.sh"

[ "$#" -gt 0 ] || {
  echo "at least one package name is required" >&2
  exit 2
}

attempts="${FORMAL_AI_APT_ATTEMPTS:-3}"
attempt_seconds="${FORMAL_AI_APT_ATTEMPT_SECONDS:-90}"
retry_delay_seconds="${FORMAL_AI_APT_RETRY_DELAY_SECONDS:-5}"
budget_seconds="${TEST_BUDGET_SECONDS:-}"
apt_get="${FORMAL_AI_APT_GET:-apt-get}"
privilege="${FORMAL_AI_APT_PRIVILEGE-sudo}"
packages="$*"

for name in FORMAL_AI_APT_ATTEMPTS:$attempts \
  FORMAL_AI_APT_ATTEMPT_SECONDS:$attempt_seconds \
  FORMAL_AI_APT_RETRY_DELAY_SECONDS:$retry_delay_seconds; do
  if ! [[ "${name#*:}" =~ ^[0-9]+$ ]]; then
    echo "${name%%:*} must be a non-negative integer, got ${name#*:}" >&2
    exit 2
  fi
done
[ "$attempts" -ge 1 ] || {
  echo "FORMAL_AI_APT_ATTEMPTS must be at least 1" >&2
  exit 2
}

# A budgeted retry gets the time left after the inter-attempt delays. Split
# that time into 1:2:4:... shares. Cumulative integer division makes the
# rounded deadlines add up exactly to the available time, so the last attempt
# gets every second not consumed by earlier probes or delays.
if [ -n "$budget_seconds" ]; then
  minimum_budget=$(( (attempts - 1) * retry_delay_seconds + 1 ))
  if [ "$budget_seconds" -lt "$minimum_budget" ]; then
    echo "::error title=apt install retry has no time for an attempt::\
${attempts} attempts need ${retry_delay_seconds}s delays plus at least 1s of \
execution time, but the step budget is ${budget_seconds}s." >&2
    exit 2
  fi
  available_attempt_seconds=$((budget_seconds - (attempts - 1) * retry_delay_seconds))
  weight_sum=0
  weight=1
  for ((i = 1; i <= attempts; i++)); do
    weight_sum=$((weight_sum + weight))
    weight=$((weight * 2))
  done
else
  available_attempt_seconds=""
  weight_sum=""
fi

status=0
previous_deadline=0
weight=1
for ((attempt = 1; attempt <= attempts; attempt++)); do
  started=$SECONDS

  if [ -n "$budget_seconds" ]; then
    cumulative_weight=$((weight * 2 - 1))
    cumulative_deadline=$((available_attempt_seconds * cumulative_weight / weight_sum))
    attempt_deadline=$((cumulative_deadline - previous_deadline))
    previous_deadline=$cumulative_deadline
    weight=$((weight * 2))
  else
    attempt_deadline="$attempt_seconds"
  fi

  # The deadline runs inside the privilege escalation so it signals apt-get
  # itself; killing an unprivileged parent could leave root's apt holding the
  # dpkg lock and turn a mirror stall into a lock error on every retry.
  attempt_command=(
    "$deadline" "$attempt_deadline" bash -c '
      set -e
      "$0" -o DPkg::Lock::Timeout=60 update -q
      "$0" -o DPkg::Lock::Timeout=60 install -y -q "$@"
    ' "$apt_get" "$@"
  )
  [ -z "$privilege" ] || attempt_command=("$privilege" "${attempt_command[@]}")

  status=0
  "${attempt_command[@]}" || status=$?
  elapsed=$((SECONDS - started))

  if [ "$status" -eq 0 ]; then
    printf 'apt install of %s succeeded on attempt %s/%s after %ss.\n' \
      "$packages" "$attempt" "$attempts" "$elapsed"
    exit 0
  fi

  echo "::warning title=apt install of ${packages} failed attempt ${attempt}/${attempts}::\
Attempt ${attempt} exited ${status} after ${elapsed}s of its ${attempt_deadline}s \
deadline. Exit 124 is the deadline itself -- a stalled mirror, transient and \
upstream; anything else is apt's own status." >&2

  [ "$attempt" -lt "$attempts" ] || break
  sleep "$retry_delay_seconds"
done

if [ -n "$budget_seconds" ]; then
  echo "::error title=apt install of ${packages} failed every attempt::\
All ${attempts} escalating attempts failed within the ${budget_seconds}s step \
budget; the last attempt had ${attempt_deadline}s. This is no longer a transient \
stall -- read the attempt warnings above for apt's own output." >&2
else
  echo "::error title=apt install of ${packages} failed every attempt::\
All ${attempts} attempts of ${attempt_seconds}s failed; the last exited \
${status}. This is no longer a transient stall -- read the attempt warnings \
above for apt's own output." >&2
fi
exit "$status"
