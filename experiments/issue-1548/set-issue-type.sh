#!/usr/bin/env bash
# Issue #1548: mark one issue with a GitHub issue type and the matching label.
#
# Usage: set-issue-type.sh <owner/repo> <number> <Bug|Feature|Task> <label> [log.tsv]
# The call is idempotent: an issue that already carries a type is left as it is
# and reported as "kept"; a label that is already present is not added twice.
set -euo pipefail

repo="$1"
number="$2"
type="$3"
label="$4"
log="${5:-/dev/stdout}"

current="$(gh api "repos/${repo}/issues/${number}" --jq '[(.type.name // ""), ([.labels[].name] | join(","))] | @tsv')"
current_type="${current%%$'\t'*}"
current_labels="${current#*$'\t'}"

if [[ -n "${current_type}" ]]; then
  type_outcome="kept:${current_type}"
else
  gh api --method PATCH "repos/${repo}/issues/${number}" -f "type=${type}" --silent
  type_outcome="set:${type}"
fi

if grep -qx "${label}" <<<"${current_labels//,/$'\n'}"; then
  label_outcome="kept:${label}"
else
  gh api --method POST "repos/${repo}/issues/${number}/labels" -f "labels[]=${label}" --silent
  label_outcome="added:${label}"
fi
printf '%s\t%s#%s\t%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${repo}" "${number}" "${type_outcome}" "${label_outcome}" >>"${log}"
