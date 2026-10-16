#!/usr/bin/env bash
# Shared typed-diagnostic consumer for actual coding-client E2E scripts.
server_log_evidence_setup() {
  RAW_SERVER_LOG="$1"
  SERVER_COMPLETION_PATH="${2:-/v1/chat/completions}"
  EVIDENCE_DIR="${ARTIFACT_DIR:-${OUT:-/tmp/formal-ai-agent-evidence-$PORT}}"
  mkdir -p "$EVIDENCE_DIR" || return 1
  EVIDENCE_DIR="$(cd "$EVIDENCE_DIR" && pwd)" || return 1
  # A new run never reuses stale exchanges from an earlier invocation.
  EVIDENCE_DIR="$(mktemp -d "$EVIDENCE_DIR/run.XXXXXX")" || return 1
  DIALOG_DIR="$EVIDENCE_DIR/dialog-logs"
  SEMANTIC_LOG="$EVIDENCE_DIR/formal-ai.semantic.log"
  export FORMAL_AI_DIALOG_LOG_DIR="$DIALOG_DIR"
  export FORMAL_AI_LOG_DIR="$EVIDENCE_DIR/native-logs"
}
project_server_log() {
  if [[ "${1:-}" == --headers-only ]]; then
    node "$ROOT/experiments/agent_cli_e2e/server-log-evidence.mjs" \
      "$RAW_SERVER_LOG" "$SEMANTIC_LOG" "$SERVER_COMPLETION_PATH"
  else
    node "$ROOT/experiments/agent_cli_e2e/server-log-evidence.mjs" \
      "$RAW_SERVER_LOG" "$SEMANTIC_LOG" "$SERVER_COMPLETION_PATH" "$DIALOG_DIR"
  fi
}
retain_server_log_evidence() {
  if [[ -f "$RAW_SERVER_LOG" && "$RAW_SERVER_LOG" != "$EVIDENCE_DIR/formal-ai.log" ]]; then
    cp "$RAW_SERVER_LOG" "$EVIDENCE_DIR/formal-ai.log" || return 1
  fi
}
