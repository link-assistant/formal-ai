#!/usr/bin/env bash
# How many commits on a pull request carry its own `Formal-AI-Pull-Request:` trailer.
#
# Issue #1085 (D2.3). `.github/workflows/self-authored-pull-request.yml` opens
# the pull request first, lets Formal AI author the change, and must not author
# it again on a re-run. The decision is made here, from the pull request's
# commits as the API reports them, entirely in jq. The first version of the
# guard was `git log "origin/$branch" --format=%B | grep -Fxq "$trailer"` under
# `set -o pipefail`: `grep -q` exits at the first match, `git log` then dies of
# SIGPIPE writing the rest of the history, and the pipeline reports failure on
# exactly the runs where the trailer was present. Every re-run therefore
# authored the task again (six duplicate commits on the second bot pull
# request, two on the third) while the guard looked correct.
#
# Usage: self-authored-commit-count.sh <pull-request-url>
# Prints the count (0 when none) and exits 0; exits 1 when the API call fails.
set -euo pipefail
url="${1:?a pull request URL is required}"
gh pr view "$url" --json commits \
  --jq --arg trailer "Formal-AI-Pull-Request: $url" \
  '[.commits[].messageBody | split("\n")[] | select(. == $trailer)] | length'
