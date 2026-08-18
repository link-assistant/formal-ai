#!/usr/bin/env bash
# pin-base-commit.sh
#
# Resolves the base branch tip ONCE per workflow run and publishes it as a job
# output, so every job that later merges the base branch merges the same commit.
#
# Usage:
#   BASE_REF=main bash scripts/pin-base-commit.sh
#
# Environment variables:
#   BASE_REF        The base branch (e.g. "main"). Required.
#   GITHUB_OUTPUT   Where the `commit=<sha>` output is written. Required on CI;
#                   when unset the commit is only printed, which is what makes
#                   this runnable locally.
#
# Issue #1017: `scripts/simulate-fresh-merge.sh` merges `origin/$BASE_REF`, and
# every job used to resolve that reference at its own start time. Jobs in a
# single run start minutes to an hour apart -- in run 31993872684 the six
# desktop packaging legs started across 62 minutes -- so one push to the base
# branch mid-run gave them different merged trees. Two failure modes followed:
# a gate could pass against a tree no other gate ever saw, and a release set
# could mix artifacts built from different sources (`linux-x64` and
# `macos-arm64` were built against 1858b3386 while `windows-arm64`, an hour
# later, was built against d1439e557, and all six shipped together).
#
# Only the macOS lane compared trees across jobs, so only there was the
# divergence visible at all; everywhere else it was silent. Resolving the tip
# here and passing the commit down turns "every job checked the same tree" into
# a property of the workflow rather than a race against the base branch.

set -euo pipefail

if [ -z "${BASE_REF:-}" ]; then
  echo "::error::BASE_REF is required" >&2
  exit 1
fi

# A shallow fetch is enough: only the tip's identity is needed here, and the
# jobs that merge it check out their own history.
git fetch --depth=1 origin "$BASE_REF"
commit="$(git rev-parse FETCH_HEAD)"

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "commit=$commit" >> "$GITHUB_OUTPUT"
fi

echo "Every job in this workflow will merge $BASE_REF = $commit"
