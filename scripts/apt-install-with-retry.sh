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
# whole wrapper is designed to finish inside the enclosing step budget --
# `FORMAL_AI_APT_ATTEMPTS * FORMAL_AI_APT_ATTEMPT_SECONDS` plus the delays
# between them, checked against `TEST_BUDGET_SECONDS` before the first attempt.
# `desktop/scripts/package-macos-with-retry.sh` is the same shape one runner
# family over: a retry is only an improvement while it can finish.
#
# Usage: apt-install-with-retry.sh <package> [package...]
#
# Environment:
#   FORMAL_AI_APT_ATTEMPTS          attempts before giving up (3)
#   FORMAL_AI_APT_ATTEMPT_SECONDS   deadline for one attempt (90)
#   FORMAL_AI_APT_RETRY_DELAY_SECONDS  pause between attempts (5)
#   TEST_BUDGET_SECONDS             enclosing step budget, checked when set
#   FORMAL_AI_APT_GET               apt-get binary (apt-get); tests point it
#                                   at a stand-in
#   FORMAL_AI_APT_PRIVILEGE         privilege escalation prefix (sudo); empty
#                                   when already root, as tests are
set -euo pipefail

# The per-attempt deadline is `scripts/run-with-deadline.sh`, not GNU `timeout`:
# macOS ships no `timeout`, so the tests that drive this wrapper on the macOS
# core slices died with `timeout: command not found` while the Linux job it
# ships on passed (issue #1021, run 32282461075). An absolute path because
# `sudo` resets PATH to its own secure_path.
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

# The guard issue #1017 pays for: a retry that cannot finish inside the budget
# above it converts a transient stall into a *terminated* step, which is the
# failure this wrapper exists to prevent. Refuse to start rather than discover
# it in CI.
worst_case=$((attempts * attempt_seconds + (attempts - 1) * retry_delay_seconds))
if [ -n "$budget_seconds" ] && [ "$worst_case" -gt "$budget_seconds" ]; then
  echo "::error title=apt install retry cannot finish inside its budget::\
${attempts} attempts of ${attempt_seconds}s plus ${retry_delay_seconds}s delays \
need ${worst_case}s, but the step budget is ${budget_seconds}s. Lower the \
attempts or the per-attempt deadline, or raise the budget with the job cap." >&2
  exit 2
fi

status=0
for ((attempt = 1; attempt <= attempts; attempt++)); do
  started=$SECONDS
  # The deadline runs *inside* the privilege escalation so it signals apt-get
  # itself; killing an unprivileged parent would leave root's apt holding the
  # dpkg lock and fail every remaining attempt with a lock error instead of the
  # stall that caused it. `DPkg::Lock::Timeout` covers the leftovers anyway.
  # shellcheck disable=SC2016  # `$0`/`$@` are the inner shell's, deliberately
  attempt_command=(
    "$deadline" "$attempt_seconds" bash -c '
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
Attempt ${attempt} exited ${status} after ${elapsed}s of its ${attempt_seconds}s \
deadline. Exit 124 is the deadline itself -- a stalled mirror, transient and \
upstream; anything else is apt's own status." >&2

  [ "$attempt" -lt "$attempts" ] || break
  sleep "$retry_delay_seconds"
done

echo "::error title=apt install of ${packages} failed every attempt::\
All ${attempts} attempts of ${attempt_seconds}s failed; the last exited \
${status}. This is no longer a transient stall -- read the attempt warnings \
above for apt's own output." >&2
exit "$status"
