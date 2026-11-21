#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
CODEX_BIN="${CODEX_BIN:-}"
CODEX_VERSION="${CODEX_VERSION:-0.144.1}"
CODEX_SANDBOX="${CODEX_SANDBOX:-workspace-write}"
CODEX_EXTRA_ARGS="${CODEX_EXTRA_ARGS:-}"
SERVER_PORT="${SERVER_PORT:-8859}"
PROXY_PORT="${PROXY_PORT:-8860}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
SCENARIO="${SCENARIO:-all}"
SERVER="${SERVER:-native}"
RUN_DIR="$(mktemp -d)"
WORKSPACE="$RUN_DIR/workspace"
SERVER_LOG="$RUN_DIR/formal-ai.log"
PROXY_STDERR="$RUN_DIR/proxy-stderr.log"
PROXY_LOG="$RUN_DIR/proxy.jsonl"
CODEX_LOG="$RUN_DIR/codex.log"
REPORT_LOG="$RUN_DIR/report-issue.log"
REPORT_RU_LOG="$RUN_DIR/report-issue-ru.log"
REPORT_SCOPE_LOG="$RUN_DIR/report-issue-scope.log"
MODEL_CATALOG="$RUN_DIR/formal-ai-model-catalog.json"

codex_cli() {
  if [ -n "$CODEX_BIN" ]; then
    "$CODEX_BIN" "$@"
  else
    bunx --package "@openai/codex@$CODEX_VERSION" codex "$@"
  fi
}

mkdir -p "$WORKSPACE"
mkdir -p "$RUN_DIR/codex"
git -C "$WORKSPACE" init -q
node "$ROOT/experiments/issue_859_codex_e2e/write-model-catalog.mjs" \
  "$MODEL_CATALOG"

server_command=("$BIN" serve)
if [ "$SERVER" = js ]; then
  server_command=(node "$ROOT/js/server/main.mjs")
fi
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 FORMAL_AI_DREAMING=0 \
  FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR=false \
  FORMAL_AI_DIALOG_LOG_DIR="$RUN_DIR/dialogs" \
  FORMAL_AI_MEMORY_PATH="$RUN_DIR/memory.lino" \
  "${server_command[@]}" --host 127.0.0.1 --port "$SERVER_PORT" \
  >"$SERVER_LOG" 2>&1 < /dev/null &
SERVER_PID=$!

"$BIN" proxy --listen "127.0.0.1:$PROXY_PORT" \
  --upstream "http://127.0.0.1:$SERVER_PORT" --log "$PROXY_LOG" --body \
  >"$PROXY_STDERR" 2>&1 < /dev/null &
PROXY_PID=$!

cleanup() {
  if [ -n "$ARTIFACT_DIR" ]; then
    mkdir -p "$ARTIFACT_DIR"
    for artifact in formal-ai.log proxy-stderr.log proxy.jsonl codex.log report-issue.log report-issue-ru.log report-issue-scope.log; do
      if [ -f "$RUN_DIR/$artifact" ]; then
        cp "$RUN_DIR/$artifact" "$ARTIFACT_DIR/$artifact"
      fi
    done
    if [ -f "$WORKSPACE/main.rs" ]; then
      cp "$WORKSPACE/main.rs" "$ARTIFACT_DIR/main.rs"
    fi
    codex_cli --version > "$ARTIFACT_DIR/codex-version.txt"
  fi
  kill "$PROXY_PID" "$SERVER_PID" 2>/dev/null || true
  wait "$PROXY_PID" "$SERVER_PID" 2>/dev/null || true
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

curl -fs --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PROXY_PORT/health" > /dev/null

run_codex() {
  local prompt="$1" output="$2" session="${3:-ephemeral}"
  local extra_args=() persistence=() resume=()
  case "$session" in
    ephemeral) persistence=(--ephemeral) ;;
    resume) resume=(resume --last) ;;
  esac
  if [ -n "$CODEX_EXTRA_ARGS" ]; then
    read -r -a extra_args <<< "$CODEX_EXTRA_ARGS"
  fi
  CODEX_HOME="$RUN_DIR/codex" FORMAL_AI_API_KEY="sk-local-demo" codex_cli exec \
    --ignore-user-config "${persistence[@]}" --color never \
    -c 'model_providers.formalai.name="formal-ai local server"' \
    -c "model_providers.formalai.base_url=\"http://127.0.0.1:$PROXY_PORT/api/openai/v1\"" \
    -c 'model_providers.formalai.env_key="FORMAL_AI_API_KEY"' \
    -c 'model_providers.formalai.wire_api="responses"' \
    -c 'model_provider="formalai"' \
    -c 'model="formal-ai"' \
    -c "model_catalog_json=\"$MODEL_CATALOG\"" \
    --skip-git-repo-check --sandbox "$CODEX_SANDBOX" --cd "$WORKSPACE" \
    "${extra_args[@]}" "${resume[@]}" \
    "$prompt" >"$output" 2>&1 < /dev/null
}

if [ "$SCENARIO" = all ] || [ "$SCENARIO" = code ]; then
  run_codex "Give me hello world program in Rust" "$CODEX_LOG"
  test -f "$WORKSPACE/main.rs"
  grep -Fq 'println!("Hello, world!")' "$WORKSPACE/main.rs"
  grep -Fq 'Let me run a compile this program for you.' "$CODEX_LOG"
  grep -Fq 'Let me run the compiled program for you.' "$CODEX_LOG"
  grep -Fq 'Hello, world!' "$CODEX_LOG"
  node "$ROOT/experiments/issue_859_codex_e2e/verify-proxy.mjs" code "$PROXY_LOG"
  echo "Codex created, compiled, and ran main.rs."
fi

if [ "$SCENARIO" = all ] || [ "$SCENARIO" = report ]; then
  run_codex "Report issue" "$REPORT_LOG"
  node "$ROOT/experiments/issue_859_codex_e2e/verify-proxy.mjs" report "$PROXY_LOG" "Report issue"
  grep -Fq '3. GitHub issue' "$REPORT_LOG"

  run_codex "зарепорти баг" "$REPORT_RU_LOG" record
  node "$ROOT/experiments/issue_859_codex_e2e/verify-proxy.mjs" report "$PROXY_LOG" "зарепорти баг"
  grep -Fq '3. Задача GitHub' "$REPORT_RU_LOG"
  run_codex "3" "$REPORT_SCOPE_LOG" resume
  node "$ROOT/experiments/issue_859_codex_e2e/verify-proxy.mjs" report "$PROXY_LOG" "3"
  grep -Fq '1. Оба журнала' "$REPORT_SCOPE_LOG"
  echo "Codex displayed both localized reporting menus and continued to context selection."
fi
