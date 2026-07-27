#!/usr/bin/env bash
# Real Agent CLI regression for capability-first local code search (issue #758).

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8782}"
AGENT="${AGENT:-agent}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
if [ -n "$ARTIFACT_DIR" ] && [[ "$ARTIFACT_DIR" != /* ]]; then
  ARTIFACT_DIR="$ROOT/$ARTIFACT_DIR"
fi
SERVER_LOG="/tmp/formal-ai-issue-758-$PORT.log"
AGENT_LOG="/tmp/agent-issue-758-$PORT.log"
WORKDIR="$(mktemp -d)"
MARKER="ISSUE_758_CAPABILITY_MARKER"

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

mkdir -p "$WORKDIR/src"
printf '%s\n' "const ROUTING_MARKER: &str = \"$MARKER\";" > "$WORKDIR/src/fixture.rs"
mkdir -p "$WORKDIR/bin"
printf '%s\n' '#!/usr/bin/env sh' 'echo "external gh disabled in issue #758 E2E" >&2' 'exit 1' \
  > "$WORKDIR/bin/gh"
chmod +x "$WORKDIR/bin/gh"
cd "$WORKDIR"

cat > opencode.json <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {"baseURL": "http://127.0.0.1:$PORT/v1", "apiKey": "local"},
      "models": {"formal-ai": {"name": "Formal AI Symbolic Production"}}
    }
  }
}
EOF

# Private, empty memory per run so this server's memory-fed planning stays
# independent of what other E2E scripts recorded into the shared
# ~/.formal-ai/memory.lino (issue #828); FORMAL_AI_DREAMING=0 stops the
# background compaction thread from mutating it mid-run.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!

if ! curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
  tail -80 "$SERVER_LOG"
  exit 1
fi

PATH="$WORKDIR/bin:$PATH" timeout 180 "$AGENT" run \
  --prompt "Search the local code for $MARKER" \
  --disable-stdin \
  --model "formal-ai/formal-ai" \
  > "$AGENT_LOG" 2>&1
RC=$?

tail -40 "$AGENT_LOG"
grep -Fq 'src/fixture.rs' "$AGENT_LOG" || RC=1
grep -Fq 'tool: "grep"' "$SERVER_LOG" || RC=1
if grep -F 'planned ToolCalls' "$SERVER_LOG" | grep -Fq 'tool: "websearch"'; then
  RC=1
fi

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$AGENT_LOG" "$ARTIFACT_DIR/agent-cli.log"
fi

if [ "$RC" -ne 0 ]; then
  echo "issue #758 Agent CLI E2E failed" >&2
  tail -100 "$SERVER_LOG"
  exit "$RC"
fi

echo "E2E OK: Agent CLI executed local grep and returned $MARKER"
