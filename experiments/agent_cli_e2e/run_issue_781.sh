#!/usr/bin/env bash
# Real multi-client research proof for issue #781. Each external CLI must drive
# Formal AI through its native protocol, observe one action at a time, execute
# one search plus three fetches, and receive the cited synthesis.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8783}"
AGENT="${AGENT:-agent}"
OPENCODE="${OPENCODE:-opencode}"
CLIENTS="${CLIENTS:-agent opencode claude codex}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
PROMPT="Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in"
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
    echo "== client log ==" >&2
    tail -120 "$client_log" >&2 2>/dev/null
  fi
  if [ -n "$server_log" ]; then
    echo "== Formal AI log ==" >&2
    tail -180 "$server_log" >&2 2>/dev/null
  fi
  exit 1
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
  },
  "mcp": {
    "issue781": {
      "type": "local",
      "command": ["node", "$ROOT/experiments/agent_cli_e2e/mock-research-mcp.mjs"],
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

cat > "$WORKDIR/claude-mcp.json" <<EOF
{
  "mcpServers": {
    "issue781": {
      "command": "node",
      "args": ["$ROOT/experiments/agent_cli_e2e/mock-research-mcp.mjs"]
    }
  }
}
EOF

run_client() {
  local client="$1"
  local client_port="$2"
  local client_dir="$WORKDIR/$client"
  local client_log="$client_dir/client.log"
  local server_log="$client_dir/formal-ai.log"
  local dialog_dir="$client_dir/dialogs"
  mkdir -p "$dialog_dir"

  # Private, empty memory per client run so this server's memory-fed planning
  # stays independent of what other E2E scripts recorded into the shared
  # ~/.formal-ai/memory.lino (issue #828); FORMAL_AI_DREAMING=0 stops the
  # background compaction thread from mutating it mid-run.
  FORMAL_AI_AGENT_MODE=1 \
    FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_DIALOG_LOG_DIR="$dialog_dir" \
    FORMAL_AI_MEMORY_PATH="$client_dir/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$client_port" > "$server_log" 2>&1 &
  CURRENT_SERVER_PID=$!

  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$client_port/health" >/dev/null 2>&1 \
    || fail "server never came up on port $client_port" "$client_log" "$server_log"

  if [ "$client" = agent ] || [ "$client" = opencode ]; then
    write_opencode_config "$client_port"
  fi

  case "$client" in
    agent)
      : > "$client_log"
      for attempt in 1 2 3; do
        echo "== Agent CLI attempt $attempt ==" >> "$client_log"
        (
          cd "$WORKDIR"
          "$AGENT" \
            --prompt "$PROMPT" \
            --disable-stdin \
            --model formal-ai/formal-ai \
            --no-summarize-session \
            --read-only
        ) >> "$client_log" 2>&1 || true
        if grep -q 'agentic_outcome: planned Final' "$server_log"; then
          break
        fi
      done
      ;;
    opencode)
      (
        cd "$WORKDIR"
        "$OPENCODE" run --model formal-ai/formal-ai "$PROMPT"
      ) > "$client_log" 2>&1 \
        || fail "OpenCode research turn failed" "$client_log" "$server_log"
      ;;
    claude)
      "$BIN" with --port "$client_port" --no-start-server --non-interactive claude -- \
        --mcp-config "$WORKDIR/claude-mcp.json" \
        --strict-mcp-config \
        --allowedTools "mcp__issue781__websearch,mcp__issue781__webfetch" \
        -- "$PROMPT" > "$client_log" 2>&1 \
        || fail "Claude research turn failed" "$client_log" "$server_log"
      ;;
    codex)
      "$BIN" with --port "$client_port" --no-start-server --non-interactive codex -- \
        --json \
        -c 'mcp_servers.issue781.command="node"' \
        -c "mcp_servers.issue781.args=[\"$ROOT/experiments/agent_cli_e2e/mock-research-mcp.mjs\"]" \
        "$PROMPT" > "$client_log" 2>&1 \
        || fail "Codex research turn failed" "$client_log" "$server_log"
      ;;
    *)
      fail "unknown issue #781 E2E client: $client" "$client_log" "$server_log"
      ;;
  esac

  local searches
  local fetches
  local posts
  searches="$(grep -c 'agentic_outcome: planned ToolCalls.*websearch' "$server_log" | tr -d ' ')"
  fetches="$(grep -c 'agentic_outcome: planned ToolCalls.*webfetch' "$server_log" | tr -d ' ')"
  posts="$(grep -c '^\[trace\] POST ' "$server_log" | tr -d ' ')"

  if [ -n "$ARTIFACT_DIR" ]; then
    mkdir -p "$ARTIFACT_DIR/$client"
    cp "$server_log" "$ARTIFACT_DIR/$client/formal-ai.log"
    cp "$client_log" "$ARTIFACT_DIR/$client/client.log"
    cp -R "$dialog_dir" "$ARTIFACT_DIR/$client/dialogs"
  fi

  [ "$searches" -ge 1 ] \
    || fail "$client never reached websearch" "$client_log" "$server_log"
  [ "$fetches" -ge 3 ] \
    || fail "$client planned only $fetches web fetches" "$client_log" "$server_log"
  [ "$posts" -ge 5 ] \
    || fail "$client returned only $posts model turns" "$client_log" "$server_log"
  grep -q 'agentic_outcome: planned Final' "$server_log" \
    || fail "$client never received the final synthesis" "$client_log" "$server_log"
  grep -q 'Source:' "$server_log" \
    || fail "$client final synthesis did not cite sources" "$client_log" "$server_log"

  echo "== issue #781 $client E2E OK: $searches search, $fetches fetches, $posts model turns =="
  tail -20 "$client_log"

  kill "$CURRENT_SERVER_PID" 2>/dev/null || true
  wait "$CURRENT_SERVER_PID" 2>/dev/null || true
  CURRENT_SERVER_PID=""
}

client_index=0
for client in $CLIENTS; do
  run_client "$client" "$((PORT + client_index))"
  client_index=$((client_index + 1))
done
