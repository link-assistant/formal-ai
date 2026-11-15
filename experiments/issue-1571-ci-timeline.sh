#!/usr/bin/env bash
# Lists failing jobs for the last N runs of a workflow on main (issue #1571 evidence collection).
# Usage: experiments/issue-1571-ci-timeline.sh "<workflow name>" [limit]
set -euo pipefail
wf="$1"; limit="${2:-15}"
gh run list --repo link-assistant/formal-ai --branch main --workflow "$wf" --limit "$limit" \
  --json databaseId,conclusion,headSha,createdAt |
  jq -r '.[] | "\(.databaseId) \(.conclusion) \(.headSha[0:9]) \(.createdAt)"' |
  while read -r id concl sha at; do
    echo "## $at $sha run $id $concl"
    gh run view "$id" --repo link-assistant/formal-ai --json jobs \
      --jq '.jobs[] | select(.conclusion=="failure" or .conclusion=="cancelled" or .conclusion=="timed_out") | "  - \(.name): \(.conclusion)"'
  done
