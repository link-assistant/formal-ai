#!/usr/bin/env bash
set -euo pipefail

BIN="${1:?Pass the selected release binary}"
ARTIFACTS="${2:?Pass the destination for the stream logs}"
PORT="${3:?Pass the local server port}"
export GROK_HOME="$HOME/.grok"
export FORMAL_AI_DIALOG_LOG_DIR="$HOME/dialog-logs" FORMAL_AI_REPORT_CLIENT=grok
mkdir -p "$ARTIFACTS" "$GROK_HOME"
cat > "$GROK_HOME/config.toml" <<EOF
[model.formal-ai]
model = "formal-ai"
base_url = "http://127.0.0.1:$PORT/api/openai/v1"
api_key = "formal-ai-local"
api_backend = "chat_completions"
context_window = 196608
max_completion_tokens = 8192

[marketplace]
default_skills_installs_purged = true
EOF

"$BIN" serve --agent-mode --host 127.0.0.1 --port "$PORT" > "$ARTIFACTS/server.log" 2>&1 < /dev/null &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true' EXIT
curl -fsS --retry 10 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$PORT/health" >/dev/null

session=100a1cc7-2483-46c2-803c-2ee707ab6c49
args=(--model formal-ai --no-memory --no-plan --no-subagents --disable-web-search --output-format streaming-json)
grok "${args[@]}" --session-id "$session" -p 'Hello' > "$ARTIFACTS/greeting.jsonl"
grok "${args[@]}" --resume "$session" -p 'Report issue' > "$ARTIFACTS/report.jsonl"
grok "${args[@]}" --resume "$session" -p 'GitHub issue' > "$ARTIFACTS/scope.jsonl"
# Automatic approval is confined to the final publication turn in the fixture.
grok "${args[@]}" --resume "$session" --always-approve -p 'Both logs' > "$ARTIFACTS/receipt.jsonl"
grep -Eq '"type":"text","data":"Hi[,!]' "$ARTIFACTS/greeting.jsonl"
grep -Fq 'GitHub issue' "$ARTIFACTS/report.jsonl"
grep -Fq 'Both logs' "$ARTIFACTS/scope.jsonl"
grep -Fq '"toolName":"run_terminal_command"' "$ARTIFACTS/receipt.jsonl"
grep -Fq 'Filed the issue on GitHub: https://github.com/link-assistant/formal-ai/issues/9999' "$ARTIFACTS/receipt.jsonl"
