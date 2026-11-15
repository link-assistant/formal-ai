#!/usr/bin/env bash
# Issue #1548: attach one issue to a parent as a native GitHub sub-issue.
#
# Usage: add-sub-issue.sh <owner/repo> <parent-number> <child-number> [log.tsv]
# The call is idempotent: a child that already has this parent is reported as
# "exists". A child that has a different parent is reported as "other-parent"
# and left untouched, because GitHub allows one parent per issue and moving an
# issue is a decision, not a side effect.
set -euo pipefail

repo="$1"
parent="$2"
child="$3"
log="${4:-/dev/stdout}"

child_id="$(gh api "repos/${repo}/issues/${child}" --jq .id)"
current_parent="$(gh api "repos/${repo}/issues/${child}/parent" --jq .number 2>/dev/null || true)"

if [[ "${current_parent}" == "${parent}" ]]; then
  outcome="exists"
elif [[ "${current_parent}" =~ ^[0-9]+$ ]]; then
  outcome="other-parent:${current_parent}"
else
  gh api --method POST "repos/${repo}/issues/${parent}/sub_issues" \
    -F "sub_issue_id=${child_id}" --silent
  outcome="added"
fi
printf '%s\t%s#%s\t%s#%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${repo}" "${parent}" "${repo}" "${child}" "${outcome}" >>"${log}"
