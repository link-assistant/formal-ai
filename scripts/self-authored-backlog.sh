#!/usr/bin/env bash
# Report every self-authored pull request still open, and fail on a stale one.
#
# Issue #1085 (D2.3). Opening a draft is the cheap half of the loop; the half
# that changes the repository is merging it, and the half that changes Formal
# AI is reading the ones that cannot be merged. Between 2026-09-08 and this
# script, seven bot pull requests were opened and none was merged.
#
# For each open `formal-ai/*` pull request this prints its age, its check
# state, and how far its base has moved underneath it -- the three things that
# decide what to do next:
#
#   green and current   -> merge it; that is a commit Formal AI wrote
#   red on its own diff -> read the session, close it, fix the meta algorithm
#   behind its base     -> the next authoring run rebases it; re-read after
#
# It exits non-zero when one has been open longer than MAX_AGE_DAYS, so the
# backlog is a red check rather than a list nobody reads.
set -euo pipefail

repo="${GITHUB_REPOSITORY:-link-assistant/formal-ai}"
max_age_days="${MAX_AGE_DAYS:-3}"
summary="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

open_prs=$(gh pr list --repo "$repo" --state open --limit 100 \
  --json number,title,headRefName,baseRefName,createdAt,isDraft,url \
  --jq '[.[] | select(.headRefName | startswith("formal-ai/"))]')

count=$(printf '%s' "$open_prs" | jq 'length')
{
  echo "## Self-authored pull requests still open"
  echo
} >> "$summary"

if [[ "$count" == 0 ]]; then
  {
    echo "None. Every draft Formal AI opened has been merged or closed with its"
    echo "defect filed, which is the only two ways one is finished."
  } >> "$summary"
  echo "no open self-authored pull requests"
  exit 0
fi

echo "| Pull request | Age | Checks | Behind base | What it is waiting for |" >> "$summary"
echo "| --- | --- | --- | --- | --- |" >> "$summary"

stale=0
now=$(date -u +%s)
for number in $(printf '%s' "$open_prs" | jq -r '.[].number'); do
  row=$(printf '%s' "$open_prs" | jq -r ".[] | select(.number == $number)")
  created=$(printf '%s' "$row" | jq -r .createdAt)
  base=$(printf '%s' "$row" | jq -r .baseRefName)
  url=$(printf '%s' "$row" | jq -r .url)
  age_days=$(( (now - $(date -u -d "$created" +%s 2>/dev/null || date -u -j -f '%Y-%m-%dT%H:%M:%SZ' "$created" +%s)) / 86400 ))

  failing=$(gh pr checks "$number" --repo "$repo" --json state \
    --jq '[.[] | select(.state == "FAILURE")] | length' 2>/dev/null || echo unknown)
  pending=$(gh pr checks "$number" --repo "$repo" --json state \
    --jq '[.[] | select(.state == "PENDING")] | length' 2>/dev/null || echo 0)
  behind=$(gh api "repos/$repo/compare/$(printf '%s' "$row" | jq -r .headRefName)...$base" \
    --jq '.ahead_by' 2>/dev/null || echo unknown)

  if [[ "$failing" == unknown ]]; then
    checks="no checks"
    waiting="a maintainer must close and reopen it, or set FORMAL_AI_BOT_TOKEN"
  elif [[ "$failing" != 0 ]]; then
    checks="$failing failing"
    waiting="read the session; if the defect is Formal AI's, file it and close this"
  elif [[ "$pending" != 0 ]]; then
    checks="$pending running"
    waiting="its own CI"
  else
    checks="green"
    waiting="**merge it**"
  fi
  if [[ "$behind" != unknown && "$behind" != 0 ]]; then
    waiting="$waiting (behind base by $behind; the next authoring run rebases it)"
  fi

  echo "| [#$number]($url) | ${age_days}d | $checks | $behind | $waiting |" >> "$summary"

  if (( age_days > max_age_days )); then
    stale=$((stale + 1))
    echo "::warning title=Self-authored pull request #$number is ${age_days} days old::$waiting"
  fi
done

{
  echo
  echo "A draft is finished in one of two ways: merged, or closed with the defect"
  echo "it exposed filed against the meta algorithm. Leaving it open is neither."
} >> "$summary"

if (( stale > 0 )); then
  echo "::error title=Self-authored backlog::$stale pull request(s) have been open longer than ${max_age_days} days. Merge them, or close them and file what they exposed (issue #1085 D2.3)."
  exit 1
fi

echo "$count open, none older than ${max_age_days} days"
