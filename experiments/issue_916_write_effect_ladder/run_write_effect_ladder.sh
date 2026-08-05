#!/usr/bin/env bash
# Drive every rung of the issue #916 write-effect ladder against a live
# `formal-ai serve --agent-mode` and record pass/fail per rung.
#
# Usage:
#   experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh
#
# Environment knobs:
#   BIN       Path to the formal-ai binary (default: target/release/formal-ai)
#   PORT      Server port (default: 8773)
#   RUNGS     Path to the rung dataset (default: alongside this script)
#   OUT       Results JSON path (default: <scriptdir>/results.json)
#   ONLY      Optional substring filter on rung id (e.g. ONLY=R916-02)
#   SANDBOX   Optional pre-existing sandbox root; created if unset
#   BASELINE  Optional prior results; enforce a stable-id, all-green ratchet
#
# Unlike the issue #840 task ladder, a rung here is judged on the workspace it
# leaves behind: the harness executes the planned `write_file` and
# `run_shell_command` calls for real inside a per-rung directory, re-runs each
# declared verification independently, and only then reads the answer. Narration
# alone can never pass a rung -- "I created the file" with no file is a failure.
#
# Setting BASELINE makes the run a gate (issue #408 ratchet): every prior rung id
# must still be present and green, and every appended rung must be green before
# the baseline may move.

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8773}"
RUNGS="${RUNGS:-$HERE/rungs.json}"
OUT="${OUT:-$HERE/results.json}"
ONLY="${ONLY:-}"
SANDBOX="${SANDBOX:-}"
CREATED_SANDBOX=0
BASELINE="${BASELINE:-}"

if [ ! -x "$BIN" ]; then
  echo "formal-ai binary not found at $BIN (build with: cargo build --release)" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for JSON handling and for the Python rungs" >&2
  exit 1
fi

if [ -z "$SANDBOX" ]; then
  SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-write-effect.XXXXXX")"
  CREATED_SANDBOX=1
fi
echo "sandbox: $SANDBOX"

# --- boot the server ---------------------------------------------------------
# BSD/macOS mktemp only substitutes a trailing XXXXXX template, so no suffix.
SERVER_LOG="$(mktemp "${TMPDIR:-/tmp}/formal-ai-write-effect-server.XXXXXX")"
# Agent mode is required for the server to emit tool calls at all; without it
# every rung measures the permission gate instead of the write effect.
FORMAL_AI_AGENT_MODE=1 "$BIN" serve --host 127.0.0.1 --port "$PORT" --agent-mode >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  if [ "$CREATED_SANDBOX" -eq 1 ] && [ -z "${SANDBOX_KEEP:-}" ]; then
    rm -rf -- "$SANDBOX"
  fi
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:$PORT/v1/models" >/dev/null 2>&1; then break; fi
  sleep 0.5
done
if ! curl -fsS "http://127.0.0.1:$PORT/v1/models" >/dev/null 2>&1; then
  echo "server never came up on port $PORT; log tail:" >&2
  tail -20 "$SERVER_LOG" >&2
  exit 1
fi

# --- drive every rung --------------------------------------------------------
python3 "$HERE/ladder.py" \
  --rungs "$RUNGS" \
  --out "$OUT" \
  --port "$PORT" \
  --sandbox "$SANDBOX" \
  --binary "$BIN" \
  --only "$ONLY" \
  --baseline "$BASELINE"
