#!/usr/bin/env bash
# Automated real-extension E2E for issue #763. A small development extension
# invokes the official command and exports its inherited config for a tool round trip.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
CODE="${CODE:-code}"
SERVER_PORT="${SERVER_PORT:-8872}"
PROXY_PORT="${PROXY_PORT:-8873}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-763/opencode-vscode-e2e}"
WORK="$(mktemp -d)"
STATE="$(mktemp -d)"
SERVER_LOG="$OUT/formal-ai.log"
PROXY_LOG="$OUT/proxy.jsonl"
SIGNAL="$OUT/extension-driver.json"
VSCODE_LOG="$OUT/vscode.log"
OPENCODE_LOG="$OUT/opencode.log"
EXPORTED_CONFIG="$OUT/opencode-config.json"
PROMPT="Use a tool to list the files in the current directory, then report the result."

cleanup() {
  kill "${proxy_pid:-}" "${server_pid:-}" 2>/dev/null || true
  rm -rf "$WORK" "$STATE"
}
trap cleanup EXIT

command -v "$CODE" >/dev/null
command -v opencode >/dev/null
test -x "$BIN" || {
  echo "build first: cargo build --bin formal-ai" >&2
  exit 2
}
mkdir -p "$OUT"
: >"$PROXY_LOG"
: >"$SIGNAL"
: >"$VSCODE_LOG"
: >"$OPENCODE_LOG"
printf '%s\n' 'OPENCODE_VSCODE_MARKER_763' >"$WORK/issue-763-marker.txt"

"$CODE" --user-data-dir "$STATE/user" --extensions-dir "$STATE/extensions" \
  --install-extension sst-dev.opencode --force >"$OUT/extension-install.log"

# Private, empty memory per run so this server's memory-fed planning stays
# independent of what other E2E scripts recorded into the shared
# ~/.formal-ai/memory.lino (issue #828); FORMAL_AI_DREAMING=0 stops the
# background compaction thread from mutating it mid-run.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORK/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$SERVER_PORT" >"$SERVER_LOG" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$SERVER_PORT/health" >/dev/null

"$BIN" proxy --listen "127.0.0.1:$PROXY_PORT" \
  --upstream "http://127.0.0.1:$SERVER_PORT" --log "$PROXY_LOG" &
proxy_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PROXY_PORT/health" >/dev/null

extension_version="$($CODE --user-data-dir "$STATE/user" --extensions-dir "$STATE/extensions" \
  --list-extensions --show-versions | grep -Ei '^sst-dev\.opencode@' | head -n 1)"
mkdir -p "$STATE/bin"
ln -s "$(command -v "$CODE")" "$STATE/bin/code"
launcher=()
if [ -z "${DISPLAY:-}" ]; then
  command -v xvfb-run >/dev/null
  launcher=(xvfb-run -a)
fi

(cd "$WORK" && env \
  PATH="$STATE/bin:$PATH" \
  OPENCODE_VSCODE_E2E_SIGNAL="$SIGNAL" \
  OPENCODE_VSCODE_E2E_PROMPT="$PROMPT" \
  "${launcher[@]}" "$BIN" with --no-start-server \
  --base-url "http://127.0.0.1:$PROXY_PORT" opencode-vscode \
  --user-data-dir "$STATE/user" \
  --extensions-dir "$STATE/extensions" \
  --extensionDevelopmentPath="$ROOT/experiments/opencode_vscode_e2e/driver" \
  --disable-workspace-trust --no-sandbox "$WORK") >"$VSCODE_LOG" 2>&1

grep -q '"caller":"vscode"' "$SIGNAL"
grep -q '"error":""' "$SIGNAL"
mkdir -p "$WORK/opencode-config"
(cd "$WORK" && env \
  FORMAL_AI_API_KEY=formal-ai \
  OPENCODE_CALLER=vscode \
  OPENCODE_CONFIG="$EXPORTED_CONFIG" \
  OPENCODE_CONFIG_DIR="$WORK/opencode-config" \
  opencode run --pure "$PROMPT") >"$OPENCODE_LOG" 2>&1
grep -q 'chat/completions' "$PROXY_LOG"
grep -Eq '"response_tool_calls":\[\{"name":' "$PROXY_LOG"
grep -q '"role":"tool"' "$SERVER_LOG"
printf '%s\n' "$extension_version" >"$OUT/extension-version.txt"
echo "issue #763 real-extension E2E passed; evidence: $OUT"
