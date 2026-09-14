#!/usr/bin/env bash
# Measure how long `scripts/run-with-deadline.sh` actually takes to kill a
# stalled command, in fractional seconds, so the overshoot can be attributed to
# a cause instead of guessed at (issue #1021).
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
for run in 1 2 3; do
  start=$(python3 -c 'import time; print(time.time())')
  status=0
  "$root/scripts/run-with-deadline.sh" 3 bash -c 'sleep 300' || status=$?
  end=$(python3 -c 'import time; print(time.time())')
  python3 -c "print(f'run $run: status=$status elapsed={$end - $start:.3f}s')"
done

# A longer deadline is where the counted bound would drift worst, so measure it
# too: the clock bound should take over and keep the overshoot inside a second.
start=$(python3 -c 'import time; print(time.time())')
status=0
"$root/scripts/run-with-deadline.sh" 10 bash -c 'sleep 300' || status=$?
end=$(python3 -c 'import time; print(time.time())')
python3 -c "print(f'10s deadline: status=$status elapsed={$end - $start:.3f}s')"

# A command that beats its deadline must keep its own status and not wait.
start=$(python3 -c 'import time; print(time.time())')
status=0
"$root/scripts/run-with-deadline.sh" 30 bash -c 'exit 7' || status=$?
end=$(python3 -c 'import time; print(time.time())')
python3 -c "print(f'fast command: status=$status elapsed={$end - $start:.3f}s')"
