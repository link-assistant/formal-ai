#!/usr/bin/env bash
# Report the share of sccache cache writes that the backend rejected.
#
# Issue #1081 (D9). `sccache --show-stats` prints `Cache write errors` as one
# row among twenty, nothing reads it, and no threshold exists -- so a run in
# which the compiler cache accepted half of what it was offered looks exactly
# like a healthy one. Measured on run 34095902681 (main, 6039c4d9), across the
# eight jobs that compiled anything:
#
#   Summarization quality ratchet      13 writes / 137 errors   91.3%
#   Task Ladder (issue #840 dataset)    3 writes / 126 errors   97.7%
#   Write-Effect Ladder                44 writes / 111 errors   71.6%
#   Question necessity ratchet        176 writes /  89 errors   33.6%
#   Build binaries and tests           62 writes /  30 errors   32.6%
#   Lint and Format Check             377 writes / 374 errors   49.8%
#   macOS Core Tests / test archive    36 writes /   1 error     2.7%
#   Test (macos-15-intel / spec)       66 writes /   0 errors    0.0%
#                                     ---------------------------------
#                                     843 writes / 868 errors   50.7%
#
# A write that fails is a crate the next run recompiles, so this is the cheapest
# available explanation for the budget pressure that issues #1017, #1021 and
# #1081 all landed on from the other end.
#
# Two hypotheses fit the shape of that table, and this script does not claim to
# choose between them:
#
#   1. Rate limiting by the GitHub Actions Cache service. Documented upstream in
#      Mozilla-Actions/sccache-action#50, where the backend answers "Request was
#      blocked due to exceeding usage of resource 'Count' in namespace ''" and
#      sccache counts it under exactly this counter.
#   2. Concurrent writers of identical keys. The five ubuntu jobs above overlap
#      between 07:35 and 07:42 and compile the same x86_64-unknown-linux-gnu
#      crates; the two macOS jobs run alone at 07:52 and 08:06, in a key space
#      nobody else is writing, and report ~0%.
#
# Both predict error rates that rise with simultaneity, which is what was
# measured -- note that the rate does NOT rise with finishing order, so "someone
# else got there first" in the sequential sense is already refuted. The evidence
# that separates them is the backend's own response, which sccache logs only
# under `SCCACHE_LOG=debug`. That is wired to the `FORMAL_AI_CI_VERBOSE`
# repository variable in `.github/actions/setup-sccache/action.yml` and is off by
# default; set it to `true` and rerun to collect the discriminating evidence.
#
# Until then this script does the one thing that was missing: it makes the
# counter visible, with a threshold, in the run that produced it.
#
# Usage: check-sccache-write-health.sh [<stats-json-file>|-]
#        With no argument it asks the running sccache server itself.
#
# Environment:
#   SCCACHE_WRITE_ERROR_WARN_PERCENT  warn at or above this share (25)
#   SCCACHE_WRITE_MIN_ATTEMPTS        stay quiet below this many attempts (20)
#   SCCACHE_HEALTH_LABEL              prefix for the annotation title ("")
#   SCCACHE_PATH                      sccache binary to ask (found on PATH)
#
# Exit status is always 0: this is a visibility audit, and a diagnostic that
# fails a step replaces the finding with a worse one.
set -uo pipefail

warn_percent="${SCCACHE_WRITE_ERROR_WARN_PERCENT:-25}"
min_attempts="${SCCACHE_WRITE_MIN_ATTEMPTS:-20}"
label="${SCCACHE_HEALTH_LABEL:-}"
source="${1:-}"

read_stats() {
  if [ -n "$source" ] && [ "$source" != "-" ]; then
    cat -- "$source" 2> /dev/null
    return 0
  fi
  if [ "$source" = "-" ]; then
    cat
    return 0
  fi
  local sccache="${SCCACHE_PATH:-}"
  [ -n "$sccache" ] || sccache="$(command -v sccache 2> /dev/null || true)"
  [ -n "$sccache" ] || return 0
  # No `| head` anywhere near sccache: an early-closing reader makes it exit on
  # `Broken pipe (os error 32)`. The same trap is documented in
  # .github/actions/setup-sccache/action.yml.
  "$sccache" --show-stats --stats-format=json 2> /dev/null
}

# The counters are flat integers in a single-line JSON object, so a targeted
# match is enough and does not add a python/jq dependency to every runner that
# compiles Rust. `"cache_writes":` and `"cache_write_errors":` are distinct keys
# and the trailing colon keeps the shorter one from matching the longer.
field() {
  local name="$1" body="$2" value
  value="$(printf '%s' "$body" | grep -o "\"${name}\":[0-9]\+" | head -n 1 | cut -d: -f2)"
  printf '%s' "${value:-}"
}

stats="$(read_stats)"
[ -n "$stats" ] || exit 0

writes="$(field cache_writes "$stats")"
errors="$(field cache_write_errors "$stats")"
if [ -z "$writes" ] || [ -z "$errors" ]; then
  exit 0
fi

attempts=$((writes + errors))
if [ "$attempts" -lt "$min_attempts" ]; then
  # Below the floor the ratio is noise, not a measurement. Issue #1081 spent a
  # day on a headroom audit that reported percentages computed from two
  # samples; the fix there and here is to say "too few" rather than a number.
  exit 0
fi

share=$((errors * 100 / attempts))
[ "$share" -ge "$warn_percent" ] || exit 0

title="Compiler cache rejected ${share}% of its writes"
[ -z "$label" ] || title="${label}: ${title}"
echo "::warning title=${title}::sccache offered ${attempts} compiled artifacts to the GitHub Actions Cache and ${errors} were refused (${writes} stored). Every refused write is a crate the next run compiles again, so this is build time the cache is not saving. Set the FORMAL_AI_CI_VERBOSE repository variable to true and rerun to capture the backend's own response (SCCACHE_LOG=debug), which is what separates rate limiting from concurrent writers of the same key -- see scripts/check-sccache-write-health.sh." >&2

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    # `printf -- ` because the format string starts with a Markdown bullet, and
    # a bare leading `-` is argument-parsed away before it reaches the output.
    printf -- '- Compiler cache writes'
    [ -z "$label" ] || printf ' (%s)' "$label"
    printf ': %s stored, %s refused, %s%% refused.\n' "$writes" "$errors" "$share"
  } >> "$GITHUB_STEP_SUMMARY"
fi

exit 0
