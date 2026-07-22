#!/usr/bin/env bash
# Real local path-discovery proof for issue #819. Agent, OpenCode, Claude Code,
# and Codex must execute one client-side `find`, return its result, and finish.
# A second OpenCode run captures its real TUI frame-by-frame through
# link-foundation/command-stream and verifies the complete dialog sequence.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8784}"
AGENT="${AGENT:-agent}"
OPENCODE="${OPENCODE:-opencode}"
CLIENTS="${CLIENTS:-agent opencode claude codex}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
PROMPT="Find hive-mind-control center folder on my desktop"
WORKDIR="$(mktemp -d)"
DESKTOP_DIR="$WORKDIR/Desktop"
EXPECTED_PATH="$DESKTOP_DIR/Archive/hive-control-center"
TUI_DIR="$ROOT/experiments/agent_cli_e2e/issue_819_tui"
CURRENT_SERVER_PID=""

cleanup() {
  if [ -n "$CURRENT_SERVER_PID" ]; then
    kill "$CURRENT_SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  local message="$1"
  local client_log="${2:-}"
  local server_log="${3:-}"
  echo "!! $message" >&2
  if [ -n "$client_log" ]; then
    echo "== client log ==" >&2
    tail -120 "$client_log" >&2 2>/dev/null
  fi
  if [ -n "$server_log" ]; then
    echo "== Formal AI log ==" >&2
    tail -180 "$server_log" >&2 2>/dev/null
  fi
  exit 1
}

start_server() {
  local server_port="$1"
  local server_log="$2"
  local dialog_dir="$3"
  FORMAL_AI_AGENT_MODE=1 \
    FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_DIALOG_LOG_DIR="$dialog_dir" \
    "$BIN" serve --host 127.0.0.1 --port "$server_port" > "$server_log" 2>&1 &
  CURRENT_SERVER_PID=$!
  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$server_port/health" >/dev/null 2>&1 \
    || fail "server never came up on port $server_port" "" "$server_log"
}

stop_server() {
  kill "$CURRENT_SERVER_PID" 2>/dev/null || true
  wait "$CURRENT_SERVER_PID" 2>/dev/null || true
  CURRENT_SERVER_PID=""
}

write_opencode_config() {
  local config_port="$1"
  cat > "$WORKDIR/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$config_port/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF
}

preserve_raw_artifacts() {
  local client="$1"
  local client_dir="$2"
  if [ -n "$ARTIFACT_DIR" ]; then
    mkdir -p "$ARTIFACT_DIR/$client"
    cp "$client_dir/client.log" "$ARTIFACT_DIR/$client/client.log"
    cp "$client_dir/formal-ai.log" "$ARTIFACT_DIR/$client/formal-ai.log"
    cp -R "$client_dir/dialogs" "$ARTIFACT_DIR/$client/dialogs"
  fi
}

preserve_sequence() {
  local client="$1"
  local sequence="$2"
  if [ -n "$ARTIFACT_DIR" ]; then
    cp "$sequence" "$ARTIFACT_DIR/$client/dialog-sequence.json"
  fi
}

run_client() {
  local client="$1"
  local client_port="$2"
  local client_dir="$WORKDIR/$client"
  local client_log="$client_dir/client.log"
  local server_log="$client_dir/formal-ai.log"
  local dialog_dir="$client_dir/dialogs"
  local sequence="$client_dir/dialog-sequence.json"
  mkdir -p "$dialog_dir"
  start_server "$client_port" "$server_log" "$dialog_dir"
  write_opencode_config "$client_port"

  case "$client" in
    agent)
      : > "$client_log"
      for attempt in 1 2 3; do
        echo "== Agent CLI attempt $attempt ==" >> "$client_log"
        (
          cd "$WORKDIR"
          FORMAL_AI_DESKTOP_DIR="$DESKTOP_DIR" "$AGENT" \
            --prompt "$PROMPT" \
            --disable-stdin \
            --model formal-ai/formal-ai \
            --no-summarize-session
        ) >> "$client_log" 2>&1 || true
        if grep -q 'agentic_outcome: planned Final' "$server_log"; then
          break
        fi
      done
      ;;
    opencode)
      (
        cd "$WORKDIR"
        FORMAL_AI_DESKTOP_DIR="$DESKTOP_DIR" "$OPENCODE" run \
          --model formal-ai/formal-ai "$PROMPT"
      ) > "$client_log" 2>&1 \
        || fail "OpenCode local find failed" "$client_log" "$server_log"
      ;;
    claude)
      FORMAL_AI_DESKTOP_DIR="$DESKTOP_DIR" \
        "$BIN" with --port "$client_port" --no-start-server --non-interactive claude -- \
          --allowedTools Bash \
          --permission-mode bypassPermissions \
          -- "$PROMPT" > "$client_log" 2>&1 \
        || fail "Claude local find failed" "$client_log" "$server_log"
      ;;
    codex)
      FORMAL_AI_DESKTOP_DIR="$DESKTOP_DIR" \
        "$BIN" with --port "$client_port" --no-start-server --non-interactive codex -- \
          --json \
          --dangerously-bypass-approvals-and-sandbox \
          "$PROMPT" > "$client_log" 2>&1 \
        || fail "Codex local find failed" "$client_log" "$server_log"
      ;;
    *)
      fail "unknown issue #819 E2E client: $client" "$client_log" "$server_log"
      ;;
  esac

  preserve_raw_artifacts "$client" "$client_dir"
  node "$TUI_DIR/verify-dialog.mjs" \
    "$dialog_dir" "$client" "$sequence" "$EXPECTED_PATH" \
    || fail "$client dialog structure was incomplete" "$client_log" "$server_log"
  grep -Fq "$EXPECTED_PATH" "$client_log" \
    || fail "$client did not display the discovered path" "$client_log" "$server_log"
  preserve_sequence "$client" "$sequence"
  echo "== issue #819 $client E2E OK: user -> find -> result -> final =="
  tail -20 "$client_log"
  stop_server
}

run_opencode_tui() {
  local tui_port="$1"
  local client_dir="$WORKDIR/opencode-tui"
  local client_log="$client_dir/client.log"
  local server_log="$client_dir/formal-ai.log"
  local dialog_dir="$client_dir/dialogs"
  local sequence="$client_dir/dialog-sequence.json"
  local transcript="$client_dir/tui-transcript.json"
  mkdir -p "$dialog_dir"
  start_server "$tui_port" "$server_log" "$dialog_dir"
  write_opencode_config "$tui_port"

  ISSUE819_TUI_COMMAND="$OPENCODE . --model formal-ai/formal-ai --prompt '$PROMPT' --auto --mini" \
    ISSUE819_TUI_CWD="$WORKDIR" \
    ISSUE819_DESKTOP_DIR="$DESKTOP_DIR" \
    ISSUE819_EXPECT_PATH="$EXPECTED_PATH" \
    ISSUE819_TUI_OUTPUT="$transcript" \
    node "$TUI_DIR/capture-opencode.mjs" > "$client_log" 2>&1 \
    || fail "OpenCode TUI transcript failed" "$client_log" "$server_log"

  node "$TUI_DIR/verify-dialog.mjs" \
    "$dialog_dir" "opencode-tui" "$sequence" "$EXPECTED_PATH" \
    || fail "OpenCode TUI dialog structure was incomplete" "$client_log" "$server_log"
  if [ -n "$ARTIFACT_DIR" ]; then
    mkdir -p "$ARTIFACT_DIR/opencode-tui"
    cp "$client_log" "$ARTIFACT_DIR/opencode-tui/client.log"
    cp "$server_log" "$ARTIFACT_DIR/opencode-tui/formal-ai.log"
    cp "$sequence" "$ARTIFACT_DIR/opencode-tui/dialog-sequence.json"
    cp "$transcript" "$ARTIFACT_DIR/opencode-tui/tui-transcript.json"
    cp -R "$dialog_dir" "$ARTIFACT_DIR/opencode-tui/dialogs"
  fi
  echo "== issue #819 OpenCode TUI OK: deduplicated frames + complete dialog =="
  stop_server
}

mkdir -p "$EXPECTED_PATH"
(cd "$TUI_DIR" && bun install --frozen-lockfile && bun test) \
  || fail "command-stream TUI regression failed"

client_index=0
for client in $CLIENTS; do
  run_client "$client" "$((PORT + client_index))"
  client_index=$((client_index + 1))
done
run_opencode_tui "$((PORT + 20))"
