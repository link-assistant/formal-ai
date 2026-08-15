#!/usr/bin/env bash
# Real gemini CLI regression for caller-framing intent hijacking (issue #907).
#
# The reported failure is the gemini CLI's own `<session_context>` block — which
# it prepends to *every* turn and which the caller cannot suppress — answering
# for the user: "Today's date is Sunday, August 2, 2026 …" fired the date intent,
# so the run emitted `run_shell_command({"command":"date"})` and never attempted
# the request behind it.
#
# The bug only exists once a real client injects that framing, so this leg drives
# the actual `gemini` CLI against `formal-ai serve --agent-mode` over the native
# Gemini routes, exactly as docs/testing/agentic-cli-tools.md prescribes. Two
# phrasings run: one that must produce the program (the report's request), and
# one that must still reach `date`, so a guard that silenced the intent
# altogether cannot pass.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8907}"
GEMINI="${GEMINI:-gemini}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
if [ -n "$ARTIFACT_DIR" ] && [[ "$ARTIFACT_DIR" != /* ]]; then
  ARTIFACT_DIR="$ROOT/$ARTIFACT_DIR"
fi
SERVER_LOG="/tmp/formal-ai-issue-907-$PORT.log"
TASK_LOG="/tmp/gemini-issue-907-task-$PORT.log"
QUESTION_LOG="/tmp/gemini-issue-907-question-$PORT.log"
WORKDIR="$(mktemp -d)"

# The report's own request, and a differently-worded question that must keep
# routing to the intent (CONTRIBUTING rule 4).
TASK="${TASK:-Write a hello world program in Python.}"
EXPECT_FILE="${EXPECT_FILE:-main.py}"
EXPECT_TEXT="${EXPECT_TEXT:-Hello, world!}"
QUESTION="${QUESTION:-what is the date?}"

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

cd "$WORKDIR"

# Isolate the CLI from any cached OAuth state and select API-key auth, so the
# run is headless and talks only to the local server
# (docs/testing/agentic-cli-tools.md § Gemini CLI).
export GEMINI_CLI_HOME="$WORKDIR/home"
mkdir -p "$GEMINI_CLI_HOME/.gemini"
printf '%s\n' '{"security":{"auth":{"selectedType":"gemini-api-key"}},"model":{"name":"formal-ai"},"tools":{"useRipgrep":false}}' \
  > "$GEMINI_CLI_HOME/.gemini/settings.json"
export HOME="$GEMINI_CLI_HOME"
export TERM=xterm-256color
export GEMINI_API_KEY="sk-local-issue-907"
export GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key
export GEMINI_CLI_TRUST_WORKSPACE=true
# The Gemini protocol is mounted under /api/gemini, the endpoint the seeded
# integration declares (data/seed/client-integrations.lino, `endpoint_gemini`)
# and the one README.md documents; the CLI appends /v1beta/models/… to it.
export GOOGLE_GEMINI_BASE_URL="http://127.0.0.1:$PORT/api/gemini"

# Private, empty memory per run (issue #828) and no background compaction, so
# this leg's planning is independent of what other E2E scripts recorded.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  tail -80 "$SERVER_LOG"
  exit 1
fi

RC=0

# Leg 1 — the report's request. The framing gemini injects must not answer it.
timeout 180 "$GEMINI" -p "$TASK" --yolo < /dev/null > "$TASK_LOG" 2>&1
tail -40 "$TASK_LOG"
LEG1_END="$(wc -l < "$SERVER_LOG")"

if [ ! -f "$WORKDIR/$EXPECT_FILE" ]; then
  echo "issue #907: $EXPECT_FILE was never written — the request was dropped" >&2
  RC=1
elif ! grep -Fq "$EXPECT_TEXT" "$WORKDIR/$EXPECT_FILE"; then
  echo "issue #907: $EXPECT_FILE does not carry $EXPECT_TEXT" >&2
  RC=1
fi

# The exact reported symptom: the date intent must never have fired for a turn
# whose only request was the task.
if grep -Fq '"command":"date"' "$TASK_LOG" || grep -Fq "'command': 'date'" "$TASK_LOG" \
  || sed -n "1,${LEG1_END}p" "$SERVER_LOG" | grep -Fq 'run_shell_command", arguments: "{\"command\":\"date\"}'; then
  echo "issue #907: the session_context framing still hijacked the turn" >&2
  RC=1
fi

# Leg 2 — a real question. Suppressing the intent altogether is not the fix.
timeout 180 "$GEMINI" -p "$QUESTION" --yolo < /dev/null > "$QUESTION_LOG" 2>&1
tail -40 "$QUESTION_LOG"

# The server executes the intent itself and answers with the command's output, so
# the evidence that the intent still fires is the planner's trace for this leg —
# the CLI transcript only carries the answer.
if ! sed -n "$((LEG1_END + 1)),\$p" "$SERVER_LOG" \
  | grep -Fq 'run_shell_command", arguments: "{\"command\":\"date\"}'; then
  echo "issue #907: asking for the date no longer routes to the shell intent" >&2
  RC=1
fi

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$TASK_LOG" "$ARTIFACT_DIR/gemini-task.log"
  cp "$QUESTION_LOG" "$ARTIFACT_DIR/gemini-question.log"
fi

if [ "$RC" -ne 0 ]; then
  echo "issue #907 gemini CLI E2E failed" >&2
  tail -120 "$SERVER_LOG"
  exit "$RC"
fi

echo "E2E OK: gemini CLI wrote $EXPECT_FILE, and asking for the date still runs date"
