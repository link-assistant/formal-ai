#!/usr/bin/env bash
# Time `npm audit` against the live advisory registry, and keep what it said.
#
# The gate in `scripts/check-javascript-dependencies.sh` has to pick two
# numbers -- how long one attempt may take, and how long the gate as a whole
# may spend on attempts that never answered -- and it has to recognise what a
# non-answer looks like. Both were guessed once and both were wrong, so this
# script is the measurement they are taken from now. Run it from the repository
# root; it prints one line per sample and leaves each run's output beside it.
set -euo pipefail

samples="${1:-3}"
workspace="${2:-desktop}"
ceiling="${3:-400}"
output_dir="$(mktemp -d)"

echo "measuring $samples samples of npm audit in $workspace (ceiling ${ceiling}s)"
for ((sample = 1; sample <= samples; sample++)); do
  began=$SECONDS
  status=0
  (cd "$workspace" && timeout "$ceiling" npm audit --package-lock-only --audit-level=moderate) \
    >"$output_dir/sample-$sample.txt" 2>&1 || status=$?
  echo "sample $sample: exit=$status elapsed=$((SECONDS - began))s"
  sed -n '1,3p' "$output_dir/sample-$sample.txt" | sed 's/^/  | /'
done
echo "full output in $output_dir"
