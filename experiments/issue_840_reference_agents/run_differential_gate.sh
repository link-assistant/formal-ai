#!/usr/bin/env bash
# Re-run Formal AI's deterministic issue #840 ladder, then compare its local
# fixture transcript with the recorded reference-agent baseline.

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
RESULTS="${OUT:-}"
REMOVE_RESULTS=0

if [ -z "$RESULTS" ]; then
  RESULTS="$(mktemp "${TMPDIR:-/tmp}/formal-ai-issue-840-differential.XXXXXX")"
  REMOVE_RESULTS=1
fi

cleanup() {
  if [ "$REMOVE_RESULTS" -eq 1 ]; then
    rm -f "$RESULTS"
  fi
}
trap cleanup EXIT

OUT="$RESULTS" \
PORT="${PORT:-8785}" \
"$ROOT/experiments/issue_840_task_ladder/run_ladder.sh"

python3 \
  "$HERE/check_differential.py" \
  "$HERE/baseline.json" \
  "$RESULTS"
