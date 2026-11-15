#!/usr/bin/env bash
# Every check-run annotation GitHub attached at one commit, as TSV:
# check_run_id, check name, level, path:line, message (first line).
# Usage: analysis/collect-annotations.sh <sha> > annotations/<name>.tsv
set -euo pipefail
sha="$1"
repo=link-assistant/formal-ai
printf 'check_run_id\tcheck\tlevel\tlocation\tmessage\n'
gh api --paginate "repos/$repo/commits/$sha/check-runs?per_page=100" \
  --jq '.check_runs[] | select(.output.annotations_count > 0) | "\(.id)\t\(.name)"' |
while IFS=$'\t' read -r id name; do
  gh api --paginate "repos/$repo/check-runs/$id/annotations?per_page=100" \
    --jq ".[] | [\"$id\", \"$name\", .annotation_level, \"\(.path):\(.start_line)\", (.message | split(\"\n\")[0])] | @tsv"
done
