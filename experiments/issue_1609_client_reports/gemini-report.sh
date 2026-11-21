#!/usr/bin/env bash
set -euo pipefail

BIN="${1:?Pass the selected release binary}"
ARTIFACTS="${2:?Pass the destination for the stream logs}"
PORT="${3:?Pass the local server port}"
mkdir -p "$ARTIFACTS" "$HOME/.gemini"
printf '%s\n' '{"security":{"auth":{"selectedType":"gemini-api-key"}},"privacy":{"usageStatisticsEnabled":false}}' > "$HOME/.gemini/settings.json"

export GEMINI_CLI_HOME="$HOME" GEMINI_API_KEY=formal-ai GEMINI_DEFAULT_AUTH_TYPE=gemini-api-key
export GEMINI_CLI_TRUST_WORKSPACE=true GOOGLE_GEMINI_BASE_URL="http://127.0.0.1:$PORT/api/gemini"
export FORMAL_AI_REPORT_CLIENT=gemini FORMAL_AI_REPORT_DIALOG_ID=gemini_release_report
export GEMINI_CLI_CUSTOM_HEADERS="x-formal-ai-dialog-id:$FORMAL_AI_REPORT_DIALOG_ID"
export FORMAL_AI_DIALOG_LOG_DIR="$HOME/dialog-logs"
"$BIN" serve --agent-mode --host 127.0.0.1 --port "$PORT" > "$ARTIFACTS/server.log" 2>&1 < /dev/null &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true' EXIT
curl -fsS --retry 10 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$PORT/health" >/dev/null

# Every invocation uses the same on-disk transcript. Automatic tool approval is
# confined to the last turn of this loopback fixture.
gemini -m formal-ai -p 'Hello' --output-format stream-json > "$ARTIFACTS/greeting.jsonl"
gemini -m formal-ai --resume latest -p 'Report issue' --output-format stream-json > "$ARTIFACTS/report.jsonl"
gemini -m formal-ai --resume latest -p 'GitHub issue' --output-format stream-json > "$ARTIFACTS/scope.jsonl"
gemini -m formal-ai --resume latest -p 'Both logs' --approval-mode yolo --output-format stream-json > "$ARTIFACTS/receipt.jsonl"
grep -Fq 'GitHub issue' "$ARTIFACTS/report.jsonl"
grep -Fq 'Both logs' "$ARTIFACTS/scope.jsonl"
grep -Fq '"tool_name":"run_shell_command"' "$ARTIFACTS/receipt.jsonl"
grep -Fq 'Filed the issue on GitHub: https://github.com/link-assistant/formal-ai/issues/9999' "$ARTIFACTS/receipt.jsonl"
