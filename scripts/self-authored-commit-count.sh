#!/usr/bin/env bash
# How many commits on a pull request carry its own `Formal-AI-Pull-Request:` trailer.
#
# Issue #1085 (D2.3). `.github/workflows/self-authored-pull-request.yml` opens
# the pull request first, lets Formal AI author the change, and must not author
# it again on a re-run. The decision is made here, from the pull request's
# commits as the API reports them.
#
# The first version of the guard was
# `git log "origin/$branch" --format=%B | grep -Fxq "$trailer"` under
# `set -o pipefail`: `grep -q` exits at the first match, `git log` then dies of
# SIGPIPE writing the rest of the history, and the pipeline reports failure on
# exactly the runs where the trailer was present. Every re-run therefore
# authored the task again (six duplicate commits on #1093, two on #1094) while
# the guard looked correct. `grep -c` reads all of its input, so it cannot
# repeat that; the trailing `|| true` is for its exit 1 on a count of zero,
# which it prints before exiting.
#
# Usage: self-authored-commit-count.sh <pull-request-url>
# Prints the count (0 when none). Exits non-zero when the API call fails, so a
# caller under `set -e` stops rather than reading a failure as "not authored".
set -euo pipefail
url="${1:?a pull request URL is required}"
bodies=$(gh pr view "$url" --json commits --jq '.commits[].messageBody')
printf '%s\n' "$bodies" | grep -c -F -x "Formal-AI-Pull-Request: $url" || true
