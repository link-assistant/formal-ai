#!/usr/bin/env bash
# Drive OpenCode's real multi-select question UI for the issue #819 report
# flow, then prove that every selected destination executes.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
OPENCODE="${OPENCODE:-opencode}"
PORT="${PORT:-8804}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
PROMPT="Report"
ISSUE_URL="https://github.com/link-assistant/formal-ai/issues/9999"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/bin"
DIALOG_DIR="$WORKDIR/dialogs"
ACTIONS_LOG="$WORKDIR/report-actions.log"
SERVER_LOG="$WORKDIR/formal-ai.log"
TRANSCRIPT="$WORKDIR/tui-transcript.json"
CLIENT_LOG="$WORKDIR/client.log"
SERVER_PID=""

cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  echo "!! $1" >&2
  tail -120 "$CLIENT_LOG" >&2 2>/dev/null
  tail -180 "$SERVER_LOG" >&2 2>/dev/null
  exit 1
}

mkdir -p "$FAKE_BIN" "$DIALOG_DIR"
cat > "$WORKDIR/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF

cat > "$FAKE_BIN/formal-ai" <<EOF
#!/usr/bin/env bash
echo "formal-ai \$*" >> "$ACTIONS_LOG"
EOF
cat > "$FAKE_BIN/curl" <<EOF
#!/usr/bin/env bash
echo "curl \$*" >> "$ACTIONS_LOG"
EOF
cat > "$FAKE_BIN/gh" <<EOF
#!/usr/bin/env bash
echo "gh \$*" >> "$ACTIONS_LOG"
echo "$ISSUE_URL"
EOF
chmod +x "$FAKE_BIN/formal-ai" "$FAKE_BIN/curl" "$FAKE_BIN/gh"

FORMAL_AI_AGENT_MODE=1 \
  FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_DIALOG_LOG_DIR="$DIALOG_DIR" \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
/usr/bin/curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
  || fail "server never came up on port $PORT"

ISSUE819_TUI_COMMAND="$OPENCODE . --model formal-ai/formal-ai --prompt '$PROMPT' --auto --mini" \
  ISSUE819_TUI_CWD="$WORKDIR" \
  ISSUE819_FORMAL_AI_BASE_URL="http://127.0.0.1:$PORT" \
  ISSUE819_TUI_PATH="$FAKE_BIN:$PATH" \
  ISSUE819_REPORT_URL="$ISSUE_URL" \
  ISSUE819_TUI_OUTPUT="$TRANSCRIPT" \
  node "$ROOT/experiments/agent_cli_e2e/issue_819_tui/capture-report.mjs" \
    > "$CLIENT_LOG" 2>&1 \
  || fail "OpenCode report TUI transcript failed"

for action in 'formal-ai context export' 'curl -fsS' 'gh issue create'; do
  grep -Fq "$action" "$ACTIONS_LOG" \
    || fail "selected report action did not execute: $action"
done

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$CLIENT_LOG" "$ARTIFACT_DIR/client.log"
  cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$ACTIONS_LOG" "$ARTIFACT_DIR/report-actions.log"
  cp "$TRANSCRIPT" "$ARTIFACT_DIR/tui-transcript.json"
  cp -R "$DIALOG_DIR" "$ARTIFACT_DIR/dialogs"
fi

echo "== issue #819 OpenCode report TUI OK: three selections, three actions =="
cat "$ACTIONS_LOG"
