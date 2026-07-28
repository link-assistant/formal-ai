#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8794}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-674/agent-cli}"
TASK='When I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation.'
EXPECTED="$ROOT/data/meta/issue-674-compiled-procedure.lino"

command -v "$AGENT" >/dev/null
command -v curl >/dev/null
command -v rg >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
[[ -f "$EXPECTED" ]] || {
  echo "missing reviewed artifact: $EXPECTED" >&2
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
git -C "$work" config user.email issue-674-agent@example.invalid
git -C "$work" config user.name issue-674-agent-fixture
touch "$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(cd "$work" && PATH="$(dirname "$BIN"):$PATH" FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
rm "$OUT/agent-stream.raw.log" "$OUT/agent-stderr.log"

cmp "$work/compiled-procedure.lino" "$EXPECTED"
cp "$work/compiled-procedure.lino" "$OUT/agent-authored-compiled-procedure.lino"
rg -q 'formal-ai procedure conformance' "$OUT/agent-stream.jsonl"
"$BIN" agent --task "$TASK" --session-json "$OUT/session.json" >/dev/null

posts="$(grep -c 'POST /api/openai/v1/chat/completions' "$OUT/formal-ai.log")"
[[ "$posts" -ge 4 ]] || {
  echo "expected at least four Agent/Formal-AI chat rounds, got $posts" >&2
  exit 1
}
session_id="$(rg -o 'ses_[A-Za-z0-9]+' "$OUT/agent-stream.jsonl" | head -n 1)"
[[ -n "$session_id" ]] || {
  echo "external Agent stream contains no session id" >&2
  exit 1
}
echo "issue 674 Agent CLI replay passed: session=$session_id posts=$posts"
