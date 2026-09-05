#!/usr/bin/env bash
# Simulate merging the latest PR base so CI validates the result that will land.

set -euo pipefail

if [ -z "${BASE_REF:-}" ]; then
  echo "BASE_REF is required" >&2
  exit 2
fi

echo "Synchronizing PR with latest $BASE_REF"

git config user.email "github-actions[bot]@users.noreply.github.com"
git config user.name "github-actions[bot]"
git fetch origin "$BASE_REF"

behind_count=$(git rev-list --count "HEAD..origin/$BASE_REF")
if [ "$behind_count" -eq 0 ]; then
  echo "Merge preview is up to date with $BASE_REF"
  exit 0
fi

echo "Base branch has $behind_count new commit(s); simulating a fresh merge"
if git merge "origin/$BASE_REF" --no-edit; then
  echo "Fresh merge simulation succeeded"
else
  echo "::error::Merge conflict detected with latest $BASE_REF" >&2
  exit 1
fi
