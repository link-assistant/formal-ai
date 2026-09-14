#!/usr/bin/env bash
# Issue #1079, defect D11: `--compaction-model same` is silently ignored.
#
# The Agent CLI documents `--compaction-model` (upstream issue #219) with a
# special value `same`, meaning "compact with the session's own model". This
# repository passed it on every wrapped invocation. It never took effect:
# `src/cli/model-config.js` resolves the *cascade* argument first and only
# falls through to the single-model argument when the cascade parses empty --
# and yargs always supplies the default cascade, which begins with the hosted
# `opencode/big-pickle`. So the fallback branch is unreachable in practice.
#
# This script proves it against a provider that is unmistakably local: a
# stdlib-only HTTP server on 127.0.0.1 answering `ok` to everything. Whatever
# model name the CLI logs as its compaction cascade, it did not come from here.
#
#   ./experiments/issue_1079_agent_compaction_flag/run.sh
#
# Environment:
#   AGENT  path to the Agent CLI (default: the `agent` on PATH)
#   OUT    output directory (default: a fresh mktemp -d)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT="${AGENT:-$(command -v agent || true)}"
OUT="${OUT:-$(mktemp -d)}"
PORT="${PORT:-8123}"
# A second instance, identical except that it answers the *non-streaming*
# completion -- the one the summarizer makes -- with HTTP 500. See the `fatal`
# case below.
FAIL_PORT="${FAIL_PORT:-$((PORT + 1))}"

if [ -z "$AGENT" ]; then
  echo "no Agent CLI found; install with: bun add -g @link-assistant/agent" >&2
  exit 2
fi

mkdir -p "$OUT"
echo "== agent: $AGENT ($("$AGENT" --version 2>&1 | head -1))"
echo "== out:   $OUT"

python3 "$HERE/mock_openai_server.py" "$PORT" &
SERVER_PID=$!
MOCK_FAIL_NON_STREAMING=1 MOCK_STREAM_DELAY="${MOCK_STREAM_DELAY:-5}" \
  python3 "$HERE/mock_openai_server.py" "$FAIL_PORT" &
FAIL_SERVER_PID=$!
trap 'kill "$SERVER_PID" "$FAIL_SERVER_PID" 2>/dev/null || true' EXIT

for port in "$PORT" "$FAIL_PORT"; do
  for _ in $(seq 1 50); do
    if curl -fsS "http://127.0.0.1:$port/v1/models" >/dev/null 2>&1; then
      break
    fi
    sleep 0.1
  done
done

mock_config() {
  printf '{"$schema":"https://opencode.ai/config.json","provider":{"mock":{"name":"mock","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/v1","apiKey":"{env:MOCK_API_KEY}"},"models":{"mock-model":{"name":"mock-model"}}}},"model":"mock/mock-model"}' \
    "$1"
}

config="$(mock_config "$PORT")"
fail_config="$(mock_config "$FAIL_PORT")"

# `run_case <label> <flag...>` records the CLI's own decision. The line to read
# is `compaction models cascade configured`, which reports both the resolved
# models and whether they came from the command line or from the default.
run_case() {
  local label="$1"
  shift
  local log="$OUT/$label.log"
  MOCK_API_KEY=mock LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    timeout 120 "$AGENT" \
      --model mock/mock-model \
      --permission-mode auto \
      --output-format stream-json \
      --compact-json \
      --disable-stdin \
      "$@" \
      --prompt 'say ok' \
    >"$log" 2>&1 || true
  printf '\n== %s: agent %s\n' "$label" "$*"
  grep -o '"message":"compaction models cascade configured"[^}]*' "$log" \
    | head -1 || true
  grep -o '"models":\[[^]]*\],"source":"[a-z]*"' "$log" | head -1 || true
  grep -o '"service":"session.summary","providerID":"[^"]*","modelID":"[^"]*"' \
    "$log" | head -1 || true
}

# `run_fatal` is the second defect: the summary is fire-and-forget, so when it
# rejects nothing catches it, `process.on('unhandledRejection')` runs, and the
# CLI exits 1 -- aborting the turn that was still streaming. The summary only
# produces a session *title*, and it takes the whole run down with it.
#
# The failing mock answers 400 rather than 500 so the rejection is not delayed
# by three rounds of backoff, and it holds the stream open for
# MOCK_STREAM_DELAY seconds so the session is still running when the rejection
# lands -- the ordering a real multi-minute turn always has, and the reason
# this looks like flakiness in CI rather than a reproducible failure.
run_fatal() {
  local log="$OUT/fatal.log"
  local status=0
  MOCK_API_KEY=mock LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$fail_config" \
    timeout 120 "$AGENT" \
      --model mock/mock-model \
      --permission-mode auto \
      --output-format stream-json \
      --compact-json \
      --disable-stdin \
      --compaction-models "(mock/mock-model)" \
      --prompt 'say ok' \
    >"$log" 2>&1 || status=$?
  printf '\n== fatal: the summarizer was refused, exit=%s\n' "$status"
  grep -o '"errorType":"UnhandledRejection","message":"[^"]*"' "$log" | head -1 \
    || true
  if grep -q '"type":"result"' "$log"; then
    echo '  (the turn reached its result event)'
  else
    echo '  no "type":"result" event: the turn was aborted mid-stream'
  fi
  FATAL_STATUS="$status"
}

run_case singular --compaction-model same
run_case plural --compaction-models "(same)"
run_fatal

echo
echo "== verdict"
singular="$(grep -o '"models":\[[^]]*\],"source":"[a-z]*"' "$OUT/singular.log" | head -1)"
plural="$(grep -o '"models":\[[^]]*\],"source":"[a-z]*"' "$OUT/plural.log" | head -1)"
printf '  --compaction-model same    -> %s\n' "$singular"
printf '  --compaction-models (same) -> %s\n' "$plural"

status=0
case "$singular" in
  *'"source":"default"'*)
    echo '  DEFECT REPRODUCED: the singular flag left the default cascade in place' ;;
  *)
    echo '  the singular flag now takes effect -- upstream fixed it'
    status=1 ;;
esac
case "$plural" in
  *'["same"],"source":"cli"'*)
    echo '  WORKAROUND HOLDS: the plural flag pins compaction to the session model' ;;
  *)
    echo '  WORKAROUND BROKEN: the plural flag no longer pins the session model'
    status=1 ;;
esac
if grep -q '"errorType":"UnhandledRejection"' "$OUT/fatal.log" \
  && [ "${FATAL_STATUS:-0}" != "0" ]; then
  printf '  DEFECT REPRODUCED: a failed session title exits %s and kills the turn\n' \
    "$FATAL_STATUS"
else
  echo '  a failed summary is no longer fatal -- upstream fixed it'
  status=1
fi
echo "  logs in $OUT"
exit "$status"
