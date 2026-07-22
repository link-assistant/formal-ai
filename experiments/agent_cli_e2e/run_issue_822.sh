#!/usr/bin/env bash
# Real Agent CLI reproduction for issue #822: reporting pauses for both user
# choices, then files complete matching context in Links Notation.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8783}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
if [[ -n "$ARTIFACT_DIR" && "$ARTIFACT_DIR" != /* ]]; then
  ARTIFACT_DIR="$ROOT/$ARTIFACT_DIR"
fi
WORKDIR="$(mktemp -d)"
FAKE_BIN="$WORKDIR/fake-bin"
SERVER_LOG="$WORKDIR/formal-ai.log"
AGENT_LOG="$WORKDIR/agent-cli.log"
GH_CAPTURE="$WORKDIR/gh-invocation.txt"
BODY_CAPTURE="$WORKDIR/issue-body.md"
GIST_CAPTURE="$WORKDIR/formal-ai-context.lino"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

mkdir -p "$FAKE_BIN"
cd "$WORKDIR"

cat > opencode.json <<EOF
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
        "formal-ai": { "name": "Formal AI" }
      }
    }
  }
}
EOF

cat > "$FAKE_BIN/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-} ${2:-}" == "gist create" ]]; then
  cp "${@: -1}" "$FORMAL_AI_GIST_CAPTURE"
  printf '%s\n' 'https://gist.github.com/formal-ai/complete-context'
  exit 0
fi

if [[ "${1:-} ${2:-}" == "issue create" ]]; then
  printf '%s\n' "$@" > "$FORMAL_AI_GH_CAPTURE"
  while [[ $# -gt 0 ]]; do
    if [[ "$1" == "--body-file" ]]; then
      cp "$2" "$FORMAL_AI_BODY_CAPTURE"
      break
    fi
    shift
  done
  printf '%s\n' 'https://github.com/link-assistant/formal-ai/issues/999999'
  exit 0
fi

printf 'unexpected gh invocation: %s\n' "$*" >&2
exit 2
EOF
chmod +x "$FAKE_BIN/gh"

# Private, empty memory per run so the chat handler's memory-fed planning and the
# `POST /v1/chat/completions` round count stay deterministic and independent of
# what earlier E2E scripts recorded into the shared ~/.formal-ai/memory.lino
# (issue #828). FORMAL_AI_DREAMING=0 stops background compaction of that file.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_DIALOG_LOG_DIR="$WORKDIR/dialog-logs" \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null

fail() {
  echo "!! $*" >&2
  echo "== Agent CLI log ==" >&2
  tail -160 "$AGENT_LOG" >&2 2>/dev/null || true
  echo "== Formal AI server log ==" >&2
  tail -240 "$SERVER_LOG" >&2 2>/dev/null || true
  exit 1
}

run_turn() {
  local label="$1"
  local prompt="$2"
  shift 2
  echo "== $label: $prompt ==" | tee -a "$AGENT_LOG"
  FORMAL_AI_BASE_URL="http://127.0.0.1:$PORT/v1" \
    FORMAL_AI_GH_CAPTURE="$GH_CAPTURE" \
    FORMAL_AI_BODY_CAPTURE="$BODY_CAPTURE" \
    FORMAL_AI_GIST_CAPTURE="$GIST_CAPTURE" \
    LINK_ASSISTANT_AGENT_DISABLE_AUTOUPDATE=1 \
    PATH="$FAKE_BIN:$PATH" \
    timeout 90 "$AGENT" run \
      --prompt "$prompt" \
      --disable-stdin \
      --model formal-ai/formal-ai \
      --no-summarize-session \
      "$@" >> "$AGENT_LOG" 2>&1 || fail "$label failed"
}

run_turn report "Report issue"
[[ ! -e "$GH_CAPTURE" ]] || fail "the first question filed an issue before confirmation"
grep -Fq 'What would you like to report?' "$AGENT_LOG" \
  || fail "the first turn did not ask what kind of report to produce"

run_turn destination "GitHub issue" --continue --no-fork
[[ ! -e "$GH_CAPTURE" ]] || fail "the destination choice filed before selecting context"
grep -Fq 'Which context should the GitHub issue include?' "$AGENT_LOG" \
  || fail "the second turn did not ask which logs to include"

run_turn contents "Both logs" --continue --no-fork

[[ -s "$GH_CAPTURE" ]] || fail "the confirmed report did not invoke gh"
grep -Fxq 'issue' "$GH_CAPTURE" || fail "gh did not receive the issue subcommand"
grep -Fxq 'create' "$GH_CAPTURE" || fail "gh did not receive the create subcommand"
grep -Fxq 'link-assistant/formal-ai' "$GH_CAPTURE" \
  || fail "gh did not target the Formal AI repository"
[[ -s "$BODY_CAPTURE" ]] || fail "the confirmed report had no body"
grep -Fq 'Complete agentic context' "$BODY_CAPTURE" \
  || grep -Fq 'Agentic context' "$BODY_CAPTURE" \
  || fail "the report body did not describe its complete context"

if [[ -s "$GIST_CAPTURE" ]]; then
  grep -Fq 'conversation' "$GIST_CAPTURE" \
    || fail "the linked context did not contain the conversation"
  grep -Fq 'server_logs' "$GIST_CAPTURE" \
    || fail "the linked context did not contain server logs"
else
  grep -Fq 'conversation' "$BODY_CAPTURE" \
    || fail "the inline context did not contain the conversation"
  grep -Fq 'server_logs' "$BODY_CAPTURE" \
    || fail "the inline context did not contain server logs"
fi

grep -Fq 'issues/999999' "$AGENT_LOG" \
  || fail "the Agent CLI did not return the created issue URL"

posts="$(grep -c 'POST /v1/chat/completions' "$SERVER_LOG" || true)"
[[ "$posts" -ge 3 ]] || fail "expected at least three confirmation rounds, got $posts"

echo "== issue #822 E2E OK: two confirmations preceded a complete-context issue ($posts rounds) =="
tail -60 "$AGENT_LOG"
if [[ -n "$ARTIFACT_DIR" ]]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$SERVER_LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$AGENT_LOG" "$ARTIFACT_DIR/agent-cli.log"
  cp "$GH_CAPTURE" "$ARTIFACT_DIR/gh-invocation.txt"
  cp "$BODY_CAPTURE" "$ARTIFACT_DIR/issue-body.md"
  if [[ -s "$GIST_CAPTURE" ]]; then
    cp "$GIST_CAPTURE" "$ARTIFACT_DIR/formal-ai-context.lino"
  fi
fi
