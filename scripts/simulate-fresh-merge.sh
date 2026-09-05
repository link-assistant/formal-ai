#!/usr/bin/env bash
# simulate-fresh-merge.sh
#
# Simulates a fresh merge of the current PR branch with the latest base branch.
# This ensures CI checks run against the actual merge result, not a stale merge preview.
#
# Usage:
#   BASE_REF=main bash scripts/simulate-fresh-merge.sh
#
# Environment variables:
#   BASE_REF       The base branch to merge with (e.g. "main"). Required.
#   FRESH_MERGE_RETRY_DELAY_SECONDS
#                  Optional. Seconds to wait between fetch attempts (default 5;
#                  the wait grows with the attempt number). Tests set it to 0.
#   BASE_COMMIT    Optional. A specific commit on the base branch to merge
#                  instead of its current tip. Set this when several jobs must
#                  reach the *same* merged tree: resolving the tip separately in
#                  each job makes the result a function of when each job started,
#                  so a push to the base branch mid-run silently gives two jobs
#                  two different trees (issue #1017 -- the macOS slices are
#                  serialized by the runner pool and start up to an hour apart,
#                  and every slice that started after such a push failed its
#                  archive tree check against an archive built before it).
#
# Exit code 0 = merge succeeded or not needed; non-zero = merge conflict detected.
#
# Adopted from link-foundation/js-ai-driven-development-pipeline-template
# (issue #808 / R3), with `$BASE_REF` quoted at every expansion -- upstream
# leaves it bare on lines 44 and 54 while quoting it on 37, so a branch name
# containing a glob character or whitespace resolved differently depending on
# which line read it (issue #812; reported upstream).
#
# GitHub checks out `refs/pull/N/merge`, a merge preview
# computed when the PR was last synced -- it does NOT include commits pushed to
# the base branch afterwards. A PR can therefore be green against a base that no
# longer exists and break `main` on merge.

set -euo pipefail

echo "=== Synchronizing PR with latest $BASE_REF ==="
echo "This prevents stale merge preview issues (a green PR that breaks main)"
echo ""

# Configure git for merge
git config user.email "github-actions[bot]@users.noreply.github.com"
git config user.name "github-actions[bot]"

# Fetch with a bounded retry. Run 33973154494 failed this step 30 seconds in
# with `fatal: unable to access ... Could not resolve host: github.com` -- name
# resolution on the runner, nothing to do with the change under test, and every
# later step skipped (issue #1076, D22). A transient network failure must cost
# a wait, not a red build; a fetch that never succeeds must still fail, because
# skipping the merge simulation silently is the false negative this check
# exists to prevent.
fetch_with_retry() {
  local attempt=1
  local max_attempts=5
  local delay="${FRESH_MERGE_RETRY_DELAY_SECONDS:-5}"

  while :; do
    if git fetch origin "$@"; then
      return 0
    fi
    if [ "$attempt" -ge "$max_attempts" ]; then
      echo "::error::git fetch origin $* failed $max_attempts times; the base branch could not be read"
      return 1
    fi
    echo "git fetch origin $* failed (attempt $attempt/$max_attempts); retrying in $((delay * attempt))s"
    sleep "$((delay * attempt))"
    attempt=$((attempt + 1))
  done
}

# Fetch the latest base branch
echo "Fetching latest $BASE_REF..."
fetch_with_retry "$BASE_REF"

# Get current and base branch info
CURRENT_SHA=$(git rev-parse HEAD)

if [ -n "${BASE_COMMIT:-}" ]; then
  # Pinned mode: merge the commit the caller names, even if the branch has moved
  # past it. Fetch it explicitly -- on a shallow or filtered clone the object is
  # not guaranteed to be present just because the branch tip is.
  fetch_with_retry "$BASE_COMMIT" 2> /dev/null || true
  BASE_SHA=$(git rev-parse "$BASE_COMMIT^{commit}")
  echo "Pinned base commit requested: $BASE_COMMIT -> $BASE_SHA"
else
  BASE_SHA=$(git rev-parse "origin/$BASE_REF")
fi

echo "Current checkout (merge preview): $CURRENT_SHA"
echo "Base branch ($BASE_REF) commit to merge: $BASE_SHA"
echo ""

# Check if the base commit has changes not in the merge preview
BEHIND_COUNT=$(git rev-list --count "HEAD..$BASE_SHA")

if [ "$BEHIND_COUNT" -eq 0 ]; then
  echo "Merge preview is up-to-date with $BASE_REF. No simulation needed."
else
  echo "Base branch has $BEHIND_COUNT new commit(s) since PR was opened/synced."
  echo "Simulating fresh merge to validate actual merge result..."
  echo ""

  # Attempt to merge the base commit resolved above
  if git merge "$BASE_SHA" --no-edit; then
    echo ""
    echo "Fresh merge simulation successful!"
    echo "Checks will now run against the up-to-date merged state."
  else
    echo ""
    echo "::error::Merge conflict detected! PR needs to be rebased/updated before it can be merged."
    echo "The PR branch is out of sync with $BASE_REF and cannot be automatically merged."
    exit 1
  fi
fi
echo ""
