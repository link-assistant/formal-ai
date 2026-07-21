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
#   BASE_REF  The base branch to merge with (e.g. "main"). Required.
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

# Fetch the latest base branch
echo "Fetching latest $BASE_REF..."
git fetch origin "$BASE_REF"

# Get current and base branch info
CURRENT_SHA=$(git rev-parse HEAD)
BASE_SHA=$(git rev-parse "origin/$BASE_REF")

echo "Current checkout (merge preview): $CURRENT_SHA"
echo "Latest base branch ($BASE_REF): $BASE_SHA"
echo ""

# Check if base branch has new commits not in the merge preview
BEHIND_COUNT=$(git rev-list --count "HEAD..origin/$BASE_REF")

if [ "$BEHIND_COUNT" -eq 0 ]; then
  echo "Merge preview is up-to-date with $BASE_REF. No simulation needed."
else
  echo "Base branch has $BEHIND_COUNT new commit(s) since PR was opened/synced."
  echo "Simulating fresh merge to validate actual merge result..."
  echo ""

  # Attempt to merge the latest base branch
  if git merge "origin/$BASE_REF" --no-edit; then
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
