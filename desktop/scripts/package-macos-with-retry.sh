#!/usr/bin/env bash
# Retry only the transient host failures observed on GitHub-hosted macOS
# runners:
#
#   1. hdiutil create/attach failing against the runner's disk-image service.
#      See https://github.com/actions/runner-images/issues/7522.
#   2. electron-builder's toolset download stalling for the whole of its
#      600 000 ms `got` request timeout. Issue #1017: the build produced a
#      complete DMG, ZIP and both blockmaps and *then* failed, because the
#      stalled request had been recorded by an AsyncTaskManager whose
#      `awaitTasks()` rethrew it after the retry underneath had already
#      succeeded. `scripts/prefetch-builder-toolsets.mjs` removes the download
#      itself; this pattern is the backstop for the archives it could not seed.
set -euo pipefail

readonly max_attempts=3
retry_delay_seconds="${FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS:-5}"
# Wall-clock ceiling for this wrapper, retries included; 0 disables the check.
# A retry that would run past the ceiling is not started: packaging is the last
# expensive step in the job, so an overrun would be killed by `timeout-minutes`,
# and GitHub reports that kill as **cancelled**, not failed — the same false
# negative issue #1017 exists to remove (D1). Failing now with electron-builder's
# own status keeps the job red for the reason it actually failed.
#
# The ceiling is normally *derived*, not guessed: the caller passes the epoch
# second by which the job must be finished (`…_DEADLINE_EPOCH`) and the wrapper
# subtracts the time already spent. A fixed number cannot do that job — the same
# packaging step starts 26 minutes into one run and 33 into another, so any
# constant is either too small to allow the hdiutil retry this wrapper was
# written for, or too large to prevent the cancellation it must prevent.
# `…_BUDGET_SECONDS` stays as an explicit override and wins when both are set.
package_budget_seconds="${FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS:-}"
package_deadline_epoch="${FORMAL_AI_MACOS_PACKAGE_DEADLINE_EPOCH:-}"

# Signatures that are worth another attempt. Everything else — signing,
# notarization, dependency and arbitrary builder failures — fails on its first
# attempt, so a retry can never turn a real defect into a green build.
readonly transient_signatures=(
  # The runner's disk-image service, documented in actions/runner-images#7522.
  'hdiutil: (create|attach) failed - (Device not configured|Resource busy|No child processes)'
  # got's request timeout inside electron-builder's toolset download (#1017).
  "Timeout awaiting 'request' for [0-9]+ms"
  # Issue #1055: the same download, dropped mid-stream instead of stalling.
  # `Build windows-x64` failed with `⨯ read ECONNRESET failedTask=build` while
  # signtool.exe fetched its toolset, on a commit whose previous run of the same
  # job on the same branch had succeeded. A reset carries no status and is not a
  # signing or dependency error, so it belongs here rather than failing outright.
  'read ECONNRESET'
  'connect ETIMEDOUT'
)

if ! [[ "$retry_delay_seconds" =~ ^[0-9]+$ ]]; then
  echo "FORMAL_AI_MACOS_PACKAGE_RETRY_DELAY_SECONDS must be a non-negative integer" >&2
  exit 2
fi
if [ -n "$package_budget_seconds" ] && ! [[ "$package_budget_seconds" =~ ^[0-9]+$ ]]; then
  echo "FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS must be a non-negative integer" >&2
  exit 2
fi
if [ -n "$package_deadline_epoch" ] && ! [[ "$package_deadline_epoch" =~ ^[0-9]+$ ]]; then
  echo "FORMAL_AI_MACOS_PACKAGE_DEADLINE_EPOCH must be a non-negative integer" >&2
  exit 2
fi
if [ -z "$package_budget_seconds" ]; then
  if [ -n "$package_deadline_epoch" ]; then
    package_budget_seconds=$((package_deadline_epoch - $(date +%s)))
    # A deadline already in the past still yields a live guard rather than a
    # disabled one: 1 second refuses every retry a failing attempt could ask for.
    if [ "$package_budget_seconds" -lt 1 ]; then
      package_budget_seconds=1
    fi
    echo "macOS packaging budget: ${package_budget_seconds}s, derived from the job deadline"
  else
    package_budget_seconds=0
  fi
elif [ "$package_budget_seconds" -gt 0 ]; then
  echo "macOS packaging budget: ${package_budget_seconds}s, set explicitly"
fi
if [ "$#" -eq 0 ]; then
  echo "usage: $0 <electron-builder arguments...>" >&2
  exit 2
fi

package_log="$(mktemp "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/formal-ai-macos-package.log.XXXXXX")"
cleanup_log() {
  rm -f -- "$package_log"
}
trap cleanup_log EXIT

attempt=1
while [ "$attempt" -le "$max_attempts" ]; do
  attempt_started="$SECONDS"
  set +e
  npx --no-install electron-builder "$@" 2>&1 | tee "$package_log"
  package_status="${PIPESTATUS[0]}"
  set -e
  attempt_seconds=$((SECONDS - attempt_started))

  if [ "$package_status" -eq 0 ]; then
    exit 0
  fi

  matched_signature=""
  for signature in "${transient_signatures[@]}"; do
    if grep -Eq "$signature" "$package_log"; then
      matched_signature="$signature"
      break
    fi
  done
  if [ -z "$matched_signature" ]; then
    exit "$package_status"
  fi
  if [ "$attempt" -eq "$max_attempts" ]; then
    echo "macOS packaging still failed after ${max_attempts} attempts" >&2
    exit "$package_status"
  fi
  # Never start an attempt the job cannot finish: the previous attempt is the
  # best available estimate of the next one's cost.
  if [ "$package_budget_seconds" -gt 0 ] &&
    [ "$((SECONDS + attempt_seconds))" -gt "$package_budget_seconds" ]; then
    echo "::warning title=macOS packaging retry skipped::Another ~${attempt_seconds}s attempt would exceed the ${package_budget_seconds}s packaging budget after ${SECONDS}s; failing with electron-builder's own status instead of risking a timeout-minutes cancellation"
    exit "$package_status"
  fi

  echo "::warning title=Transient macOS packaging failure::Retrying electron-builder after a transient failure (${matched_signature}) on attempt ${attempt}/${max_attempts}"
  # Preserve completed ZIP/app output while removing only an incomplete
  # top-level disk image that would otherwise collide with the next attempt.
  if [ -d release ]; then
    find release -maxdepth 1 -type f -name '*.dmg' -delete
  fi
  sync
  sleep_seconds=$((retry_delay_seconds * attempt))
  if [ "$sleep_seconds" -gt 0 ]; then
    sleep "$sleep_seconds"
  fi
  attempt=$((attempt + 1))
done
