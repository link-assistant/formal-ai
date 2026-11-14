#!/usr/bin/env bash
# Issue #1548: publish one native GitHub "blocked by" relationship.
#
# Usage: add-blocked-by.sh <dependent> <blocker> [log.tsv]
#   <dependent> and <blocker> are "owner/repo#number".
# The prerequisite (blocker) blocks the dependent. The call is idempotent:
# an edge that already exists is reported as "exists" and left untouched.
set -euo pipefail

dependent="$1"
blocker="$2"
log="${3:-/dev/stdout}"

dependent_repo="${dependent%#*}"
dependent_number="${dependent#*#}"
blocker_repo="${blocker%#*}"
blocker_number="${blocker#*#}"

blocker_id="$(gh api "repos/${blocker_repo}/issues/${blocker_number}" --jq .id)"

existing="$(gh api --paginate "repos/${dependent_repo}/issues/${dependent_number}/dependencies/blocked_by" --jq '.[].id')"
if grep -qx "${blocker_id}" <<<"${existing}"; then
  outcome="exists"
else
  gh api --method POST \
    "repos/${dependent_repo}/issues/${dependent_number}/dependencies/blocked_by" \
    -F "issue_id=${blocker_id}" --silent
  outcome="added"
fi
printf '%s\t%s\t%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${dependent}" "${blocker}" "${outcome}" >>"${log}"
