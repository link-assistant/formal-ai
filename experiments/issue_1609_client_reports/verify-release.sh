#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$(realpath "${1:?Pass the selected release binary}")"
test -x "$BIN"
WORKDIR="$(mktemp -d)"
trap 'rm -rf -- "$WORKDIR"' EXIT
ARTIFACT_DIR="${ARTIFACT_DIR:-$WORKDIR/evidence}"
mkdir -p "$ARTIFACT_DIR"
ARTIFACT_DIR="$(realpath "$ARTIFACT_DIR")"

cd "$ROOT"
FORMAL_AI_TEST_NATIVE_CLI="$BIN" node --test --test-concurrency=1 \
  rust/tests/web/issue-1609-client-reports.test.mjs | tee "$ARTIFACT_DIR/protocol.log"

# Use the published clients and the normal wrapper invocation. The machine has
# no provider credentials and each client gets its own writable configuration.
unset CODEX_HOME OPENAI_API_KEY ANTHROPIC_API_KEY ANTHROPIC_AUTH_TOKEN FORMAL_AI_API_KEY
unset FORMAL_AI_DIALOG_LOG_DIR FORMAL_AI_LOG_DIR FORMAL_AI_LOG_LEVEL FORMAL_AI_LOG_STDERR FORMAL_AI_SILENT
unset GH_TOKEN GITHUB_TOKEN GH_ENTERPRISE_TOKEN GITHUB_ENTERPRISE_TOKEN
export FORMAL_AI_DREAMING=0 FORMAL_AI_ALLOW_NETWORK=0 OPENCODE_DISABLE_AUTOUPDATE=true
export CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 DISABLE_AUTOUPDATER=1 LINK_ASSISTANT_AGENT_DISABLE_AUTOUPDATE=1
for client in claude opencode codex agent gemini; do
  for language in en ru; do
    home="$WORKDIR/$client-$language"
    mkdir -p "$home"/{data,config,cache,state}
    if [[ "$language" == en ]]; then prompt=Hello; expected='Hi[,!]'; else prompt=Привет; expected=Здравствуйте; fi
    log="$ARTIFACT_DIR/$client-$language.log"
    # The default port is set per client, and the wrapper starts and stops it.
    # Retain its normal background diagnostics under the isolated home.
    (
      cd "$home"
      HOME="$home" XDG_DATA_HOME="$home/data" XDG_CONFIG_HOME="$home/config" \
        XDG_CACHE_HOME="$home/cache" XDG_STATE_HOME="$home/state" \
        FORMAL_AI_MEMORY_PATH="$home/memory.lino" \
        "$BIN" with --port 19409 "$client" "$prompt" </dev/null
    ) >"$log" 2>&1
    grep -Eq "$expected" "$log"
    if grep -Fq '(formal-ai-log-v1' "$log" || grep -Fq 'Cannot satisfy' "$log"; then
      cat "$log" >&2
      exit 1
    fi
    grep -rq 'started%20a%20temporary%20server' "$home/.formal-ai/logs"
    find "$home/dialog-logs" -name '*.conversation.lino' -size +0c -print -quit | grep -q .
  done
done

for client in claude agent codex gemini; do
  home="$WORKDIR/$client-report"
  mkdir -p "$home"/{bin,data,config,cache,state}
  ln -s "$BIN" "$home/bin/formal-ai"
  install -m 755 "$ROOT/experiments/issue_1609_client_reports/fake-gh.sh" "$home/bin/gh"
  (
    cd "$home"
    export HOME="$home" XDG_DATA_HOME="$home/data" XDG_CONFIG_HOME="$home/config"
    export XDG_CACHE_HOME="$home/cache" XDG_STATE_HOME="$home/state"
    export PATH="$home/bin:$PATH" FORMAL_AI_MEMORY_PATH="$home/memory.lino"
    export FORMAL_AI_TEST_ISSUE_BODY="$ARTIFACT_DIR/$client-issue-body.md"
    case "$client" in
      claude)
        node "$ROOT/experiments/issue_1609_client_reports/stream-report.mjs" \
          claude "$BIN" with --port 19409 claude
        ;;
      agent)
        # Let the client consume the stream from stdin directly.
        node "$ROOT/experiments/issue_1609_client_reports/stream-report.mjs" \
          agent "$BIN" with --interactive --port 19409 agent
        ;;
      codex)
        # New CI containers can deny Bubblewrap network namespace setup. This
        # fixture runs against loopback and uses the recording gh above.
        BIN="$BIN" CODEX_BIN="$(command -v codex)" CODEX_SANDBOX=danger-full-access \
          SCENARIO=report-publication SERVER_PORT=19409 PROXY_PORT=19410 \
          ARTIFACT_DIR="$ARTIFACT_DIR/codex-report" RUST_LOG=warn \
          bash "$ROOT/experiments/issue_859_codex_e2e/run.sh"
        ;;
      gemini)
        bash "$ROOT/experiments/issue_1609_client_reports/gemini-report.sh" \
          "$BIN" "$ARTIFACT_DIR/gemini-report" 19409
        ;;
    esac
  ) > "$ARTIFACT_DIR/$client-report.log" 2>&1
  grep -Eq 'harness_message_count: [1-9][0-9]*$' "$ARTIFACT_DIR/$client-issue-body.md"
  grep -Eq 'server_message_count: [1-9][0-9]*$' "$ARTIFACT_DIR/$client-issue-body.md"
done

# Existing PTY replay exercises report selection, every export, and the final
# URL with stock OpenCode. Only the GitHub publication boundary is replaced.
BIN="$BIN" PORT=19409 ARTIFACT_DIR="$ARTIFACT_DIR/opencode-report" \
  bash experiments/agent_cli_e2e/run_issue_819_report_tui.sh
