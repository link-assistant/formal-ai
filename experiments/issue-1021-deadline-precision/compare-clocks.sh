#!/usr/bin/env bash
# Reproduce the two earlier drafts of `scripts/run-with-deadline.sh` and measure
# all three against the same 3s deadline, so finding 17's numbers can be checked
# rather than believed (issue #1021).
#
# Draft 1 polled once a second and read elapsed time from bash's `SECONDS`.
# Draft 2 sharpened the poll to 0.1s, which exposed what `SECONDS` had been
# hiding: it is a difference of whole-second clock readings, so it can read a
# second high and expire the deadline early.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

seconds_only() { # $1 poll interval -> a copy of the script with the old clock
  sed -e 's/^  elapsed_tenths=$(((SECONDS - $1 - 1) \* 10))$/  elapsed_tenths=$(((SECONDS - $1) * 10))/' \
    -e '/^  \[ "\$elapsed_tenths" -gt "\$2" \] || elapsed_tenths="\$2"$/d' \
    "$root/scripts/run-with-deadline.sh" > "$work/draft-$1.sh"
  echo "$work/draft-$1.sh"
}

measure() { # $1 label, $2 script, $3 poll interval
  local start end status=0
  start=$(python3 -c 'import time; print(time.time())')
  FORMAL_AI_DEADLINE_POLL_SECONDS="$3" bash "$2" 3 bash -c 'sleep 300' || status=$?
  end=$(python3 -c 'import time; print(time.time())')
  python3 -c "print(f'{\"$1\":38} status=$status  3s deadline expired after {$end - $start:.3f}s')"
}

# Repeat, because the `SECONDS` defect is a race with the whole-second boundary:
# a single run can land on the safe side of it and look correct.
draft_one=$(seconds_only 1)
draft_two=$(seconds_only 2)
for repeat in 1 2 3 4 5; do
  measure "draft 1: 1s poll, SECONDS clock" "$draft_one" 1
  measure "draft 2: 0.1s poll, SECONDS clock" "$draft_two" 0.1
  measure "shipped: 0.1s poll, both bounds" "$root/scripts/run-with-deadline.sh" 0.1
  echo
done
