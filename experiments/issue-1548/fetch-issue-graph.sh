#!/usr/bin/env bash
# Issue #1548: snapshot every open issue with its type, labels, parent,
# sub-issues and native "blocked by" / "blocking" links.
#
# Usage: fetch-issue-graph.sh <owner> <repo> <output.json>
# The output is one JSON array ordered by issue number.
set -euo pipefail

owner="$1"
repo="$2"
out="$3"

read -r -d '' query <<'GRAPHQL' || true
query($owner:String!,$repo:String!,$endCursor:String){
  repository(owner:$owner,name:$repo){
    issues(first:50,states:OPEN,after:$endCursor){
      pageInfo{hasNextPage endCursor}
      nodes{
        number title id
        issueType{name}
        parent{number state}
        subIssues(first:100){nodes{number state}}
        blockedBy(first:50){nodes{number state repository{nameWithOwner}}}
        blocking(first:50){nodes{number state repository{nameWithOwner}}}
        labels(first:20){nodes{name}}
      }
    }
  }
}
GRAPHQL

gh api graphql --paginate -F owner="${owner}" -F repo="${repo}" -f query="${query}" \
  --jq '.data.repository.issues.nodes[]' | jq -s 'sort_by(.number)' >"${out}"
jq -r '"open issues: \(length); without a type: \([.[] | select(.issueType == null)] | length)"' "${out}"
