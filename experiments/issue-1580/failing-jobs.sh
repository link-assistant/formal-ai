#!/usr/bin/env bash
# Lists the failing jobs of the given workflow runs and the failing test names
# found in each job log. Usage: failing-jobs.sh OUT_DIR RUN_ID...
set -euo pipefail
repo=link-assistant/formal-ai
out=$1; shift
mkdir -p "$out"
for run in "$@"; do
  gh run view "$run" --repo "$repo" --json jobs \
    --jq '.jobs[] | select(.conclusion=="failure" or .conclusion=="timed_out" or .conclusion=="cancelled") | "\(.databaseId)\t\(.conclusion)\t\(.name)"' |
  while IFS=$'\t' read -r job conclusion name; do
    log="$out/job-$job.log"
    [ -s "$log" ] || gh api --allow-escape-sequences "repos/$repo/actions/jobs/$job/logs" | sed 's/\x1b\[[0-9;]*m//g' > "$log" || true
    echo "run=$run job=$job $conclusion :: $name"
    grep -oE "test [^ ]+ \.\.\. FAILED|##\[error\][^\n]{0,200}|not ok [0-9]+ - .{0,150}" "$log" | sort -u | head -25 | sed 's/^/    /'
  done
done
