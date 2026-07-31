#!/usr/bin/env bash
# Run issue #707's ten deterministic computer-use plans twice through the real
# @link-assistant/agent CLI and Formal AI's HTTP + MCP surfaces.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8907}"
AGENT_TIMEOUT_SECONDS="${AGENT_TIMEOUT_SECONDS:-120}"
EVIDENCE_DIR="${EVIDENCE_DIR:-$ROOT/docs/case-studies/issue-707/agent-cli-evidence/computer-use}"
WORKDIR="$(mktemp -d)"
SERVER_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  echo "!! $1" >&2
  if [[ -n "$SERVER_PID" ]]; then
    tail -120 "$WORKDIR/${CURRENT_PHASE:-record}/server.log" >&2 2>/dev/null || true
  fi
  exit 1
}

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --bin formal-ai" >&2
  exit 2
}

mapfile -t TASK_IDS < <(sed -n 's/^  task //p' "$ROOT/data/seed/computer-use-tasks.lino")
mapfile -t TASK_PROMPTS < <(sed -n 's/^    prompt en "\(.*\)"$/\1/p' "$ROOT/data/seed/computer-use-tasks.lino")
[[ "${#TASK_IDS[@]}" -eq 10 ]] || fail "expected exactly ten seeded task ids"
[[ "${#TASK_PROMPTS[@]}" -eq 10 ]] || fail "expected exactly ten English prompts"

mkdir -p "$EVIDENCE_DIR"
for phase in record replay; do
  mkdir -p "$EVIDENCE_DIR/$phase" "$WORKDIR/$phase"
  CURRENT_PHASE="$phase"
  cat >"$WORKDIR/$phase/opencode.json" <<EOF
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
        "formal-ai": { "name": "Formal AI verified computer use" }
      }
    }
  },
  "mcp": {
    "formal_ai": {
      "type": "remote",
      "url": "http://127.0.0.1:$PORT/mcp",
      "enabled": true,
      "tool_call_timeout": 120000
    }
  },
  "mcp_defaults": {
    "tool_call_timeout": 120000,
    "max_tool_call_timeout": 600000
  }
}
EOF

  : >"$EVIDENCE_DIR/$phase/audit.jsonl"
  FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=0 \
    FORMAL_AI_COMPUTER_USE_AUDIT_PATH="$EVIDENCE_DIR/$phase/audit.jsonl" \
    FORMAL_AI_MEMORY_PATH="$WORKDIR/$phase/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$PORT" \
    >"$WORKDIR/$phase/server.log" 2>&1 &
  SERVER_PID=$!
  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
    || fail "$phase server never became healthy"

  for index in "${!TASK_IDS[@]}"; do
    task_id="${TASK_IDS[$index]}"
    prompt="${TASK_PROMPTS[$index]}"
    log="$EVIDENCE_DIR/$phase/$task_id.jsonl"
    echo "== $phase $((index + 1))/10: $task_id =="
    (
      cd "$WORKDIR/$phase"
      timeout "$AGENT_TIMEOUT_SECONDS" "$AGENT" \
        --prompt "$prompt" \
        --mcp-default-tool-call-timeout 120000 \
        --mcp-max-tool-call-timeout 600000 \
        --disable-stdin \
        --model formal-ai/formal-ai \
        --no-summarize-session \
        --compaction-model same \
        --output-format stream-json \
        --compact-json
    ) >"$log" 2>&1 || fail "Agent CLI failed for $phase/$task_id"
    grep -q '"session_id":"ses_' "$log" \
      || fail "Agent CLI did not preserve a session id for $phase/$task_id"
    grep -q "computer_use_complete" "$log" \
      || fail "Formal AI did not complete $phase/$task_id"
  done

  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""
done

node "$ROOT/experiments/agent_cli_e2e/verify_issue_707.mjs" \
  "$ROOT/data/seed/computer-use-tasks.lino" "$EVIDENCE_DIR"

echo "== issue #707 real Agent CLI record/replay passed =="
echo "evidence: $EVIDENCE_DIR"
