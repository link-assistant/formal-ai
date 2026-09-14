#!/usr/bin/env bash
# Drive OpenCode's real multi-select question UI for the issue #819 report
# flow, then prove that every selected destination executes.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
OPENCODE="${OPENCODE:-opencode}"
PORT="${PORT:-8804}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
ISSUE_URL="https://github.com/link-assistant/formal-ai/issues/9999"
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/bin"
DIALOG_DIR="$WORKDIR/dialogs"
ACTIONS_LOG="$WORKDIR/report-actions.log"
SERVER_LOG="$WORKDIR/formal-ai.log"
ISSUE_BODY="$WORKDIR/issue-body.md"
GIST_DIR="$WORKDIR/gists"
TERMINAL_DIR="$WORKDIR/terminal"
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
  preserve_artifacts
  echo "!! $1" >&2
  tail -120 "$CLIENT_LOG" >&2 2>/dev/null
  tail -180 "$SERVER_LOG" >&2 2>/dev/null
  exit 1
}

preserve_artifacts() {
  if [ -z "$ARTIFACT_DIR" ]; then
    return
  fi
  mkdir -p "$ARTIFACT_DIR"
  for source in "$CLIENT_LOG" "$SERVER_LOG" "$ACTIONS_LOG" "$ISSUE_BODY"; do
    if [ -f "$source" ]; then
      cp "$source" "$ARTIFACT_DIR/$(basename "$source")"
    fi
  done
  for source in "$TERMINAL_DIR" "$DIALOG_DIR" "$GIST_DIR"; do
    if [ -d "$source" ]; then
      local destination
      destination="$ARTIFACT_DIR/$(basename "$source")"
      mkdir -p "$destination"
      cp -R "$source/." "$destination/"
    fi
  done
}

mkdir -p "$FAKE_BIN" "$DIALOG_DIR" "$GIST_DIR"
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
exec "$BIN" "\$@"
EOF
cat > "$FAKE_BIN/gh" <<EOF
#!/usr/bin/env bash
echo "gh \$*" >> "$ACTIONS_LOG"
if [ "\${1:-} \${2:-}" = "gist create" ]; then
  filename=''
  source_file=''
  while [ "\$#" -gt 0 ]; do
    if [ "\$1" = "--filename" ]; then
      filename="\$2"
      shift 2
      continue
    fi
    source_file="\$1"
    shift
  done
  [ -n "\$filename" ] && [ -n "\$source_file" ]
  cp "\$source_file" "$GIST_DIR/\$filename"
  printf 'https://gist.github.com/formal-ai/%s\n' "\$filename"
  exit 0
fi
while [ "\$#" -gt 0 ]; do
  if [ "\$1" = "--body-file" ]; then
    cp "\$2" "$ISSUE_BODY"
    break
  fi
  shift
done
echo "$ISSUE_URL"
EOF
chmod +x "$FAKE_BIN/formal-ai" "$FAKE_BIN/gh"

FORMAL_AI_AGENT_MODE=1 \
  FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_DIALOG_LOG_DIR="$DIALOG_DIR" \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
/usr/bin/curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
  || fail "server never came up on port $PORT"

ISSUE819_TUI_EXECUTABLE="$OPENCODE" \
  ISSUE819_TUI_CWD="$WORKDIR" \
  ISSUE819_TUI_PATH="$FAKE_BIN:$PATH" \
  ISSUE819_REPORT_URL="$ISSUE_URL" \
  ISSUE819_TUI_ARTIFACT_DIR="$TERMINAL_DIR" \
  FORMAL_AI_DIALOG_LOG_DIR="$DIALOG_DIR" \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" \
  FORMAL_AI_DREAMING=0 \
  node "$ROOT/experiments/agent_cli_e2e/issue_819_tui/capture-report.mjs" \
    > "$CLIENT_LOG" 2>&1 \
  || fail "OpenCode report TUI transcript failed"

for action in \
  '--source harness' \
  '--source server' \
  '--source both' \
  'gh gist create --filename formal-ai-harness-context-' \
  'gh gist create --filename formal-ai-server-context-' \
  'gh gist create --filename context.lino' \
  'gh issue create'; do
  grep -Fq -- "$action" "$ACTIONS_LOG" \
    || fail "selected report action did not execute: $action"
done
for provenance in '- **Session**:' '- **Context source**: both' '- **Surface**: agentic-cli'; do
  grep -Fq -- "$provenance" "$ISSUE_BODY" \
    || fail "the resulting issue body omitted report provenance: $provenance"
done
grep -Fq '## Reproduction of dialog' "$ISSUE_BODY" \
  || fail "the resulting issue body omitted the exported conversation"

shopt -s nullglob
harness_files=("$GIST_DIR"/formal-ai-harness-context-*.lino)
server_files=("$GIST_DIR"/formal-ai-server-context-*.lino)
[ "${#harness_files[@]}" -eq 1 ] \
  || fail "expected one captured harness context"
[ "${#server_files[@]}" -eq 1 ] \
  || fail "expected one captured server context"
[ -s "$GIST_DIR/context.lino" ] \
  || fail "expected one captured merged context"

grep -Fq 'conversation' "${harness_files[0]}" \
  || fail "the harness context did not contain the conversation"
if ! grep -Fq 'server_logs' "${server_files[0]}"; then
  grep -Fq 'context_export_failure' "${server_files[0]}" \
    && grep -Fq 'source server' "${server_files[0]}" \
    || fail "the server context was neither exported nor given an explicit diagnostic"
fi
grep -Fq 'conversation' "$GIST_DIR/context.lino" \
  || fail "the merged context did not contain the conversation"

for heading in '### Harness context' '### Server context' '### Merged context'; do
  grep -Fq "$heading" "$ISSUE_BODY" \
    || fail "the resulting issue body omitted $heading"
done
for context_file in "${harness_files[0]}" "${server_files[0]}" "$GIST_DIR/context.lino"; do
  grep -Fq "https://gist.github.com/formal-ai/${context_file##*/}" "$ISSUE_BODY" \
    || fail "the resulting issue body omitted the ${context_file##*/} link"
done
if grep -Fq '```lino' "$ISSUE_BODY"; then
  fail "the linked context was duplicated inline"
fi

preserve_artifacts

echo "== issue #819 OpenCode report TUI OK: selections, three links, and issue body =="
cat "$ACTIONS_LOG"
