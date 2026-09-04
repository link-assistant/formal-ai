#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8918}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-918/agent-cli-evidence}"
TASK="Create file minimal-core-invariant.md containing Compiled code belongs in Formal AI's minimal core only when it implements the meta algorithm, link-store substrate, a generic interpreter, or a host surface; domain knowledge and policy belong in data."
EXPECTED="Compiled code belongs in Formal AI's minimal core only when it implements the meta algorithm, link-store substrate, a generic interpreter, or a host surface; domain knowledge and policy belong in data."

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --bin formal-ai" >&2
  exit 2
}

mkdir -p "$OUT"
work="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue-918.XXXXXX")"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
printf '%s\n' '# Minimal core fixture' >"$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture
printf '%s\n' "$TASK" >"$OUT/task.txt"

# Keep Formal AI's generated memory, indexes, locks, and dialog logs under
# .git so the external Agent CLI does not treat binary link indexes as edits.
FORMAL_AI_AGENT_MODE=1 \
FORMAL_AI_TRACE_REQUESTS=1 \
FORMAL_AI_MEMORY_PATH="$work/.git/formal-ai-memory/memory.lino" \
FORMAL_AI_DREAMING=0 \
"$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai-server.log" 2>&1 &
server_pid=$!

curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(
  printf \
    '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' \
    "$PORT"
)"

(
  cd "$work"
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" \
    --model formalai/formal-ai \
    --permission-mode auto \
    --output-format stream-json \
    --compact-json \
    --disable-stdin \
    --prompt "$TASK"
) >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log"

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"

result="$work/minimal-core-invariant.md"
[[ -f "$result" ]] || {
  echo "Agent CLI did not create minimal-core-invariant.md" >&2
  exit 1
}
printf '%s' "$EXPECTED" | cmp -s - "$result" || {
  echo "Agent CLI created unexpected documentation bytes" >&2
  diff -u <(printf '%s' "$EXPECTED") "$result" >&2 || true
  exit 1
}

cp "$result" "$OUT/minimal-core-invariant.md"
session_id="$(grep -o '"session_id":"ses_[^"]*"' "$OUT/agent-stream.raw.log" | tail -1 | cut -d'"' -f4)"
[[ -n "$session_id" ]] || {
  echo "Agent CLI stream did not preserve a session id" >&2
  exit 1
}
printf '%s\n' "$session_id" >"$OUT/session-id.txt"
git -C "$work" diff --check
git -C "$work" status --short >"$OUT/worktree-status.txt"
echo "issue 918 Agent CLI documentation leaf passed: $session_id"
