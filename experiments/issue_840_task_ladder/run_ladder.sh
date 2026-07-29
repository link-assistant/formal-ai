#!/usr/bin/env bash
# Drive every node of the issue #840 task ladder against a live `formal-ai serve`
# and record pass/fail per node, so the whole 24-task dataset can be re-measured
# after each change instead of re-reported by hand.
#
# Usage:
#   experiments/issue_840_task_ladder/run_ladder.sh
#
# Environment knobs:
#   BIN        Path to the release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT       Server port (default: 8771)
#   TASKS      Path to the ladder dataset (default: alongside this script)
#   OUT        Results JSON path (default: <scriptdir>/results.json)
#   ONLY       Optional substring filter on task id (e.g. ONLY=838 or ONLY=838.L4)
#   SANDBOX    Optional pre-existing sandbox dir; created and populated if unset
#   MODE       `http` (default) or `tui` for the real OpenCode interface
#   TUI_ARTIFACT_DIR  Transcript/frames/cast/SVG root in TUI mode
#   REQUIRE_ALL_PASS  Exit nonzero when a selected TUI node fails (default: 0)
#   FIXTURES   Offline web corpus (default: web_fixtures.json; `none` disables)
#   BASELINE   Optional prior results; enforce a stable-ID, all-green ratchet
#   LEARNING_OUT  Review-gated failure proposals (default: beside OUT)
#
# A node passes only when its answer, route, and command-shape assertions all
# hold. Generic refusals and capability-menu fallbacks always fail. Raw tool
# results remain in the transcript as evidence but cannot satisfy answer claims.
#
# HTTP mode exits 0: this is a measurement harness, not a gate. TUI mode also
# exits 0 unless REQUIRE_ALL_PASS=1. Setting BASELINE makes HTTP mode a gate:
# every prior stable task id must remain and pass, and every appended task must
# pass before the baseline may move. Read results.json for the full evidence.

set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8771}"
TASKS="${TASKS:-$HERE/tasks.json}"
OUT="${OUT:-$HERE/results.json}"
ONLY="${ONLY:-}"
SANDBOX="${SANDBOX:-}"
CREATED_SANDBOX=0
MODE="${MODE:-http}"
TUI_ARTIFACT_DIR="${TUI_ARTIFACT_DIR:-$OUT.artifacts}"
FIXTURES="${FIXTURES:-$HERE/web_fixtures.json}"
BASELINE="${BASELINE:-}"
LEARNING_OUT="${LEARNING_OUT:-${OUT%.json}-learning.json}"

if [ ! -x "$BIN" ]; then
  echo "formal-ai binary not found at $BIN (build with: cargo build --release)" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for JSON handling" >&2
  exit 1
fi
if [ "$MODE" = "tui" ] && ! command -v opencode >/dev/null 2>&1; then
  echo "opencode is required for MODE=tui" >&2
  exit 1
fi

# --- sandbox reproducing the #838 desktop layout -----------------------------
if [ -z "$SANDBOX" ]; then
  SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-ladder.XXXXXX")"
  CREATED_SANDBOX=1
fi
python3 - "$TASKS" "$SANDBOX" <<'PY'
import json, os, sys
tasks, sandbox = sys.argv[1], sys.argv[2]
spec = json.load(open(tasks))["sandbox"]
for d in spec.get("dirs", []):
    os.makedirs(os.path.join(sandbox, d), exist_ok=True)
for f in spec.get("files", []):
    path = os.path.join(sandbox, f)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "a").close()
PY
echo "sandbox: $SANDBOX"

# --- boot the server ---------------------------------------------------------
# BSD/macOS mktemp only substitutes a trailing XXXXXX template, so no suffix.
SERVER_LOG="$(mktemp "${TMPDIR:-/tmp}/formal-ai-ladder-server.XXXXXX")"
# Agent mode is required for the server to emit tool calls at all; without it
# every local-search node fails on "Running shell commands requires Agent mode"
# and measures the permission gate instead of the routing/answer quality.
FORMAL_AI_AGENT_MODE=1 "$BIN" serve --host 127.0.0.1 --port "$PORT" --agent-mode >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
cleanup_server() { kill "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true; }
cleanup() {
  cleanup_server
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

if [ "$MODE" = "tui" ]; then
  TUI_DIR="$ROOT/experiments/agent_cli_e2e/issue_819_tui"
  (cd "$TUI_DIR" && bun install --frozen-lockfile) || exit 1
  ISSUE840_TASKS="$TASKS" \
    ISSUE840_OUT="$OUT" \
    ISSUE840_ONLY="$ONLY" \
    ISSUE840_SANDBOX="$SANDBOX" \
    ISSUE840_PORT="$PORT" \
    ISSUE840_ARTIFACT_DIR="$TUI_ARTIFACT_DIR" \
    node "$TUI_DIR/capture-ladder.mjs"
  exit $?
fi

# --- drive every node --------------------------------------------------------
python3 "$HERE/ladder.py" \
  --tasks "$TASKS" \
  --out "$OUT" \
  --port "$PORT" \
  --only "$ONLY" \
  --sandbox "$SANDBOX" \
  --fixtures "$FIXTURES" \
  --baseline "$BASELINE" \
  --learning-out "$LEARNING_OUT"
