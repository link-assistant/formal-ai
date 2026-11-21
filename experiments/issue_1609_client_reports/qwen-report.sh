#!/usr/bin/env bash
set -euo pipefail

ARTIFACTS="${1:?Pass the destination for the stream logs}"
PORT="${2:?Pass the local server port}"
shift 2
mkdir -p "$ARTIFACTS" "$HOME/.qwen"
cat > "$HOME/.qwen/settings.json" <<EOF
{
  "outboundCorrelation": { "allowDynamicHeaderValues": true },
  "modelProviders": { "openai": [{
    "id": "formal-ai", "baseUrl": "http://127.0.0.1:$PORT/api/openai/v1", "envKey": "OPENAI_API_KEY",
    "generationConfig": { "customHeaders": { "x-formal-ai-dialog-id": "\${session_id}" } }
  }] },
  "tools": { "approvalMode": "auto-edit", "eager": ["run_shell_command"] },
  "telemetry": { "enabled": false }
}
EOF
export OPENAI_API_KEY=formal-ai OPENAI_BASE_URL="http://127.0.0.1:$PORT/api/openai/v1" OPENAI_MODEL=formal-ai
export FORMAL_AI_REPORT_CLIENT=qwen FORMAL_AI_DIALOG_LOG_DIR="$HOME/dialog-logs"
"$@" --agent-mode --host 127.0.0.1 --port "$PORT" > "$ARTIFACTS/server.log" 2>&1 < /dev/null &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true' EXIT
curl -fs --retry 10 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$PORT/health" >/dev/null

SESSION=100a1cc7-2483-46c2-803c-2ee707ab6c17
trace=()
if [[ "${CAPTURE_QWEN_REQUESTS:-0}" == 1 ]]; then
  trace=(--openai-logging --openai-logging-dir "$ARTIFACTS/requests")
fi
# Only the known report command may bypass an interactive approval in this
# fixture, which uses loopback and the recording GitHub executable.
qwen --session-id "$SESSION" -p 'Hello' --output-format stream-json "${trace[@]}" > "$ARTIFACTS/greeting.jsonl"
qwen --resume "$SESSION" -p 'Report issue' --output-format stream-json "${trace[@]}" > "$ARTIFACTS/report.jsonl"
qwen --resume "$SESSION" -p 'GitHub issue' --output-format stream-json "${trace[@]}" > "$ARTIFACTS/scope.jsonl"
qwen --resume "$SESSION" -p 'Both logs' --output-format stream-json --allowed-tools 'run_shell_command(formal-ai report create-issue:*)' "${trace[@]}" > "$ARTIFACTS/receipt.jsonl"
grep -Fq 'GitHub issue' "$ARTIFACTS/report.jsonl"
grep -Fq 'Both logs' "$ARTIFACTS/scope.jsonl"
grep -Fq 'Filed the issue on GitHub: https://github.com/link-assistant/formal-ai/issues/9999' "$ARTIFACTS/receipt.jsonl"
test -s "$FORMAL_AI_DIALOG_LOG_DIR/$SESSION.conversation.lino"
