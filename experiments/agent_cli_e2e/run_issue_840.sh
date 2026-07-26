#!/usr/bin/env bash
# Self-application proof for issue #840. The existing multi-client/TUI local
# journey covers #838; two additional real Agent CLI runs cover #827's
# definition/follow-up and #826's decomposed comparison.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8784}"
AGENT="${AGENT:-agent}"
CLIENTS="${CLIENTS:-agent opencode claude codex}"
RUN_TUI="${RUN_TUI:-1}"
AGENT_TIMEOUT_SECONDS="${AGENT_TIMEOUT_SECONDS:-60}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
WORKDIR="$(mktemp -d)"
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
    tail -120 "$client_log" >&2 2>/dev/null
  fi
  if [ -n "$server_log" ]; then
    tail -180 "$server_log" >&2 2>/dev/null
  fi
  exit 1
}

local_artifacts=""
if [ -n "$ARTIFACT_DIR" ]; then
  local_artifacts="$ARTIFACT_DIR/local-search"
fi
BIN="$BIN" PORT="$PORT" AGENT="$AGENT" CLIENTS="$CLIENTS" RUN_TUI="$RUN_TUI" \
  ARTIFACT_DIR="$local_artifacts" \
  "$ROOT/experiments/agent_cli_e2e/run_issue_819.sh" \
  || fail "issues #819/#840 local-search E2E failed"

write_config() {
  local server_port="$1"
  cat > "$WORKDIR/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$server_port/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  },
  "mcp": {
    "issue840": {
      "type": "local",
      "command": ["node", "$ROOT/experiments/agent_cli_e2e/mock-grounded-action-mcp.mjs"],
      "enabled": true
    }
  },
  "tools": {
    "websearch": false,
    "webfetch": false
  }
}
EOF
}

run_agent_case() {
  local case_name="$1"
  local case_port="$2"
  local prompt="$3"
  local expected="$4"
  local expected_searches="$5"
  local expected_fetches="$6"
  local case_dir="$WORKDIR/$case_name"
  local client_log="$case_dir/agent-cli.log"
  local server_log="$case_dir/formal-ai.log"
  local dialog_dir="$case_dir/dialogs"
  mkdir -p "$dialog_dir"

  FORMAL_AI_AGENT_MODE=1 \
    FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_DIALOG_LOG_DIR="$dialog_dir" \
    FORMAL_AI_MEMORY_PATH="$case_dir/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$case_port" > "$server_log" 2>&1 &
  CURRENT_SERVER_PID=$!
  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$case_port/health" >/dev/null 2>&1 \
    || fail "$case_name server never came up" "$client_log" "$server_log"
  write_config "$case_port"

  : > "$client_log"
  for attempt in 1 2 3; do
    echo "== Agent CLI attempt $attempt ==" >> "$client_log"
    (
      cd "$WORKDIR"
      timeout "$AGENT_TIMEOUT_SECONDS" "$AGENT" \
        --prompt "$prompt" \
        --disable-stdin \
        --model formal-ai/formal-ai \
        --no-summarize-session \
        --read-only
    ) >> "$client_log" 2>&1 || true
    if grep -q 'agentic_outcome: planned Final' "$server_log"; then
      break
    fi
  done

  local searches
  local fetches
  searches="$(grep -c 'agentic_outcome: planned ToolCalls.*websearch' "$server_log" | tr -d ' ')"
  fetches="$(grep -c 'agentic_outcome: planned ToolCalls.*webfetch' "$server_log" | tr -d ' ')"
  [ "$searches" -ge "$expected_searches" ] \
    || fail "$case_name planned only $searches searches" "$client_log" "$server_log"
  [ "$fetches" -ge "$expected_fetches" ] \
    || fail "$case_name planned only $fetches fetches" "$client_log" "$server_log"
  grep -Fq "$expected" "$client_log" \
    || fail "$case_name final answer omitted $expected" "$client_log" "$server_log"
  grep -q 'agentic_outcome: planned Final' "$server_log" \
    || fail "$case_name never reached a final synthesis" "$client_log" "$server_log"

  if [ "$case_name" = "definition-followup" ]; then
    for artifact in 'развернуть всё' 'Что такое рок' 'exeAll'; do
      if grep -Fq "$artifact" "$client_log"; then
        fail "$case_name leaked source furniture: $artifact" "$client_log" "$server_log"
      fi
    done
  fi

  if [ -n "$ARTIFACT_DIR" ]; then
    mkdir -p "$ARTIFACT_DIR/$case_name"
    cp "$client_log" "$ARTIFACT_DIR/$case_name/agent-cli.log"
    cp "$server_log" "$ARTIFACT_DIR/$case_name/formal-ai.log"
    mkdir -p "$ARTIFACT_DIR/$case_name/dialogs"
    cp -R "$dialog_dir/." "$ARTIFACT_DIR/$case_name/dialogs/"
  fi

  echo "== issue #840 $case_name Agent CLI E2E OK: $searches searches, $fetches fetches =="
  tail -20 "$client_log"
  kill "$CURRENT_SERVER_PID" 2>/dev/null || true
  wait "$CURRENT_SERVER_PID" 2>/dev/null || true
  CURRENT_SERVER_PID=""
}

run_agent_case \
  "definition-followup" "$((PORT + 30))" \
  "Что такое фуфломицин? Затем: так что это такое то?" \
  "эффективност" 1 3
run_agent_case \
  "comparison" "$((PORT + 31))" \
  "ФБС vs ФБО" \
  "ФБО evidence" 2 0

report_artifacts=""
if [ -n "$ARTIFACT_DIR" ]; then
  report_artifacts="$ARTIFACT_DIR/report"
fi
BIN="$BIN" \
AGENT="$AGENT" \
PORT="$((PORT + 32))" \
REPORT_PROMPT="Зарепорти баг" \
ARTIFACT_DIR="$report_artifacts" \
  "$ROOT/experiments/issue_714_agentic_mode/run_report_e2e.sh"
