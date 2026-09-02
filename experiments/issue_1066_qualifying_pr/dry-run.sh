#!/usr/bin/env bash
# Measure what the release gate still refuses once acceptance item 3 is met.
#
# Issue #1066's third acceptance item asks for a *merged* pull request in the
# open release cycle whose every introduced non-merge commit is validly
# attributed. No branch can satisfy it: the merge is the thing being asked for,
# and CONTRIBUTING forbids adding the trailers to commits that were not authored
# through the loop. So the question this harness answers is the next one --
# *if* that pull request existed, would `Auto Release` go green?
#
# It builds the answer in a throwaway clone. Nothing here is pushed, nothing is
# committed to this repository, and the trailers it writes name a pull request
# number that does not exist. The clone is deleted on the way out.
#
# Usage: experiments/issue_1066_qualifying_pr/dry-run.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EVIDENCE="docs/case-studies/issue-924/incremental-self-authorship"
# The session that authored the committed evidence bundle, read from the bundle
# rather than pasted, so the trailer and the evidence cannot drift apart.
SESSION="$(grep -h -m1 -o 'ses_[A-Za-z0-9]*' "$ROOT/$EVIDENCE/dispatch-report.json")"
PR_NUMBER="${PR_NUMBER:-99999}"

CLONE="$(mktemp -d)/repo"
cleanup() { rm -rf "$(dirname "$CLONE")"; }
trap cleanup EXIT

git clone --quiet --no-hardlinks "$ROOT" "$CLONE"
cd "$CLONE"
git config user.email "dry-run@example.invalid"
git config user.name "issue-1066 dry run"

base="$(git rev-parse --abbrev-ref HEAD)"
git switch --quiet --create qualifying-pull-request

# The authored change stands in for whatever the real qualifying pull request
# carries. Its size is the variable this harness exists to vary: pass
# AUTHORED_LINES to see how much authored work the ratchet actually demands.
authored="${AUTHORED_LINES:-8}"
mkdir -p .agent-dry-run
: > .agent-dry-run/authored.txt
for line in $(seq 1 "$authored"); do
  printf 'authored line %s\n' "$line" >> .agent-dry-run/authored.txt
done
git add .agent-dry-run/authored.txt
git commit --quiet -m "feat: an authored change standing in for the qualifying pull request

Formal-AI-Session: $SESSION
Formal-AI-Evidence: $EVIDENCE
Formal-AI-Pull-Request: https://github.com/link-assistant/formal-ai/pull/$PR_NUMBER"

git switch --quiet "$base"
git merge --quiet --no-ff qualifying-pull-request \
  -m "Merge pull request #$PR_NUMBER from link-assistant/qualifying-pull-request"

echo "== the cycle, measured =="
rust-script "$ROOT/scripts/self-hosting-metric.rs" --since "$(git describe --tags --match 'v[0-9]*' --abbrev=0 HEAD)"
echo
echo "== the release preflight, with a qualifying pull request in the cycle =="
set +e
rust-script "$ROOT/scripts/check-self-development-release.rs"
status=$?
set -e
echo "exit=$status"
exit "$status"
