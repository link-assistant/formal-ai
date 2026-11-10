#!/usr/bin/env bash
# Issue #1548: create one planning issue with an issue type and a kind label.
#
# Usage: create-planned-issue.sh <owner/repo> <Bug|Feature|Task> <kind-label> <title> <body-file> [log.tsv] [extra-label]
# The optional extra label is applied at creation time. For link-assistant/formal-ai
# pass no-formal-ai-draft, which the issue-opened draft workflow honours, so that
# opening several planning issues at once does not start a draft build for each.
# The call is idempotent by title: an open issue with exactly the same title is
# reused and reported as "kept". The issue number is printed on stdout.
set -euo pipefail

repo="$1"
type="$2"
label="$3"
title="$4"
body_file="$5"
log="${6:-/dev/stderr}"
extra_label="${7:-}"

existing="$(gh api --paginate "repos/${repo}/issues?state=open&per_page=100" \
  --jq ".[] | select(.pull_request == null) | select(.title == \"${title//\"/\\\"}\") | .number" | head -n 1)"
if [[ -n "${existing}" ]]; then
  number="${existing}"
  outcome="kept"
else
  labels=(--label "${label}")
  if [[ -n "${extra_label}" ]]; then
    labels+=(--label "${extra_label}")
  fi
  url="$(gh issue create --repo "${repo}" --title "${title}" --body-file "${body_file}" "${labels[@]}")"
  number="${url##*/}"
  gh api --method PATCH "repos/${repo}/issues/${number}" -f "type=${type}" --silent
  outcome="created"
fi
printf '%s\t%s#%s\t%s\t%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${repo}" "${number}" "${outcome}" "${type}" "${title}" >>"${log}"
printf '%s\n' "${number}"
