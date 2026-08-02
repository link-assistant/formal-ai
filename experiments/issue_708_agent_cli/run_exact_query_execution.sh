#!/usr/bin/env bash
# Execute issue #708's exact SQL and GraphQL task through the real Agent CLI,
# with Formal AI itself serving as the model and memory-query executor.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8719}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-708/self-hosting-query-languages/execution}"
FIXTURE="$ROOT/experiments/issue_708_agent_cli/query-memory.lino"
SQL="SELECT id, content, accessCount FROM memory WHERE conversationId = 'issue-708-fixture' ORDER BY accessCount DESC LIMIT 1"
GRAPHQL='query { memoryAggregate(where: { conversationId: { eq: "issue-708-fixture" } }) { count accesses: sum(field: accessCount) } }'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
[[ -f "$FIXTURE" ]] || {
  echo "missing fixture: $FIXTURE" >&2
  exit 2
}

mkdir -p "$OUT"
work="$(mktemp -d)"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

git -C "$work" init -q
git -C "$work" config user.email issue-708-agent@example.invalid
git -C "$work" config user.name issue-708-agent-fixture
cp "$FIXTURE" "$work/memory.lino"

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"

run_query() {
  local label="$1" prompt="$2"
  (
    cd "$work"
    FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
      "$AGENT" --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin --prompt "$prompt"
  ) >"$OUT/$label-stream.raw.log" 2>"$OUT/$label-stderr.log"
  "$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/$label-stderr.log"
  grep '^{' "$OUT/$label-stream.raw.log" >"$OUT/$label-stream.jsonl"
}

run_query sql "$SQL"
run_query graphql "$GRAPHQL"

rg -q 'issue-708-alpha' "$OUT/sql-stream.jsonl"
rg -q 'memory_query_result' "$OUT/sql-stream.jsonl"
rg -q 'accessCount' "$OUT/sql-stream.jsonl"
rg -q 'memory_query_result' "$OUT/graphql-stream.jsonl"
rg -q 'count' "$OUT/graphql-stream.jsonl"
rg -q 'accesses' "$OUT/graphql-stream.jsonl"
rg -q 'integer:2' "$OUT/graphql-stream.jsonl"

posts="$(rg -c 'POST /api/openai/v1/chat/completions' "$OUT/formal-ai.log")"
[[ "$posts" -ge 2 ]] || {
  echo "expected at least two Agent/Formal-AI chat rounds, got $posts" >&2
  exit 1
}
sql_session="$(rg -o 'ses_[A-Za-z0-9]+' "$OUT/sql-stream.jsonl" | head -n 1)"
graphql_session="$(rg -o 'ses_[A-Za-z0-9]+' "$OUT/graphql-stream.jsonl" | head -n 1)"
[[ -n "$sql_session" && -n "$graphql_session" ]] || {
  echo "Agent CLI streams are missing session ids" >&2
  exit 1
}

printf '%s\n' "$SQL" >"$OUT/sql-query.txt"
printf '%s\n' "$GRAPHQL" >"$OUT/graphql-query.txt"
echo "issue 708 Agent CLI exact-query execution passed: sql=$sql_session graphql=$graphql_session posts=$posts"
