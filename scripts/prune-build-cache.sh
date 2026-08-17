#!/usr/bin/env bash
# Prune `target/` down to the artifacts of the most recent build.
#
# Cargo never removes anything: every branch, every dependency version and every
# incremental session leaves artifacts behind, and this repository's debug tree
# reaches several gigabytes within a few days of ordinary work. On CI the same
# growth is saved into the actions cache and restored on the next run, so the
# cache slowly fills with builds nobody will use again.
#
# "Most recent build" means artifacts newer than a reference point. Callers that
# know when their build started should pass that marker file as `$1`
# (`scripts/cargo-test.sh` does); with no argument the newest fingerprint cargo
# wrote is used instead, which is the best available stand-in for "this build".
#
# Set `CARGO_TEST_NO_PRUNE=1` to skip pruning entirely.
#
# Usage:
#   scripts/prune-build-cache.sh [marker-file]
set -euo pipefail

if [ -n "${CARGO_TEST_NO_PRUNE:-}" ]; then
  echo "prune-build-cache: skipped (CARGO_TEST_NO_PRUNE)"
  exit 0
fi

if [ ! -d target ]; then
  echo "prune-build-cache: no target/ directory, nothing to prune"
  exit 0
fi

marker=${1:-}
cleanup_marker=""
if [ -z "$marker" ]; then
  # No caller-supplied start time. Use the newest fingerprint cargo just wrote:
  # everything the current build touched is at least that new, and everything
  # older belongs to a build that no longer exists.
  newest=$(find target -name '.fingerprint' -prune -o -type f -newer Cargo.toml -print 2>/dev/null | head -1 || true)
  marker=$(mktemp)
  cleanup_marker=$marker
  if [ -n "$newest" ]; then
    touch -r "$newest" "$marker"
  else
    # Nothing newer than Cargo.toml: treat the whole tree as current and only
    # drop artifacts older than the manifest.
    touch -r Cargo.toml "$marker"
  fi
fi
# shellcheck disable=SC2064  # expand now: the path must survive this scope
[ -n "$cleanup_marker" ] && trap "rm -f '$cleanup_marker'" EXIT

before=$(du -sk target 2>/dev/null | cut -f1 || echo 0)

# Only build outputs are pruned. Binaries, test executables and cargo's own
# bookkeeping stay, so the next build still links rather than starting cold.
find target -type f ! -newer "$marker" \
  \( -path '*/incremental/*' -o -name '*.rlib' -o -name '*.rmeta' -o -name '*.o' \) \
  -delete 2>/dev/null || true
find target -type d -empty -delete 2>/dev/null || true

after=$(du -sk target 2>/dev/null | cut -f1 || echo 0)
freed=$(((before - after) / 1024))

if [ "$freed" -gt 0 ]; then
  echo "prune-build-cache: freed ${freed}MB (target/ is now $((after / 1024))MB)"
else
  echo "prune-build-cache: nothing stale to remove (target/ is $((after / 1024))MB)"
fi
