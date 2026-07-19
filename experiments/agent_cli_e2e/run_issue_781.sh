#!/usr/bin/env bash
# Real Agent CLI ↔ Formal AI proof for issue #781. A compatibility question must
# fan one search result set out to several independent fetches. Final synthesis
# is covered deterministically because Agent CLI 0.25.0 exits after executing a
# tool response whose finish reason it recovers as `unknown` (upstream #249).

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8783}"
AGENT="${AGENT:-agent}"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
LOG="/tmp/formal-ai-serve-$PORT.log"
AGENT_LOG="/tmp/agent-out-$PORT.log"
WORKDIR="$(mktemp -d)"

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
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; rm -rf "$WORKDIR"' EXIT

fail() {
  echo "!! $*" >&2
  echo "== Agent CLI log ==" >&2
  tail -120 "$AGENT_LOG" >&2 2>/dev/null
  echo "== Formal AI log ==" >&2
  tail -180 "$LOG" >&2 2>/dev/null
  exit 1
}

curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
  || fail "server never came up on port $PORT"

timeout 180 "$AGENT" \
  --prompt "Подбери совместимое зарядное устройство для Acer Aspire 3 A325-45 в Amazon India и проверь характеристики по независимым веб-источникам?" \
  --disable-stdin \
  --model formal-ai/formal-ai \
  --no-summarize-session \
  --read-only > "$AGENT_LOG" 2>&1 \
  || fail "Agent CLI research turn failed"

searches="$(grep -o 'tool: "websearch"' "$LOG" | wc -l | tr -d ' ')"
fetches="$(grep -o 'tool: "webfetch"' "$LOG" | wc -l | tr -d ' ')"
executed_fetches="$(grep -c '"tool": "webfetch"' "$AGENT_LOG" | tr -d ' ')"

[ "$searches" -ge 1 ] || fail "the question never reached websearch"
[ "$fetches" -ge 3 ] || fail "expected at least three webfetch plans, got $fetches"
[ "$executed_fetches" -ge 3 ] \
  || fail "expected the Agent CLI to execute three webfetches, got $executed_fetches events"

# The research loop now deepens across rounds, so it can plan a second search
# and further fetches. Its bounding invariant is that a source already read is
# never read again — but that deliberately is *not* asserted here, and the
# reason is worth recording so nobody adds the check back.
#
# The planner is stateless: it re-derives the whole plan from the transcript on
# every turn, so each traced request repeats all of the previous turns' tool
# calls. A URL therefore appears in these logs once per remaining turn plus once
# per execution, and a repeated URL in the trace is indistinguishable from a
# replayed history. Any count-based no-repeat check over this output reports
# convergence failures that did not happen.
#
# The invariant is real and is covered where it can actually be observed: the
# planner-level regression `a_source_already_read_is_not_read_again` in
# tests/unit/issue_781.rs asserts it against the planner directly.

if [ -n "$ARTIFACT_DIR" ]; then
  mkdir -p "$ARTIFACT_DIR"
  cp "$LOG" "$ARTIFACT_DIR/formal-ai.log"
  cp "$AGENT_LOG" "$ARTIFACT_DIR/agent-cli.log"
fi

echo "== issue #781 E2E OK: $searches search, $fetches planned fetches, $executed_fetches execution events =="
tail -40 "$AGENT_LOG"
