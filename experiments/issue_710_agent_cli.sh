#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8710}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-710/agent-cli-evidence/verdict-contract}"
TASK='Create file verdict-definition.md containing A works-now verdict requires a passing regression test against the current production path; a still-broken verdict requires an open focused tracking issue.'
EXPECTED='A works-now verdict requires a passing regression test against the current production path; a still-broken verdict requires an open focused tracking issue.'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}

mkdir -p "$OUT"
work="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue-710.XXXXXX")"
server_log="$OUT/formal-ai-server.log"
agent_stream="$OUT/agent-stream.raw.log"
agent_stderr="$OUT/agent-stderr.log"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
printf '%s\n' '# Issue 710 verdict fixture' >"$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture
printf '%s\n' "$TASK" >"$OUT/task.txt"

FORMAL_AI_AGENT_MODE=1 \
FORMAL_AI_TRACE_REQUESTS=1 \
FORMAL_AI_MEMORY_PATH="$work/memory.lino" \
FORMAL_AI_DREAMING=0 \
"$BIN" serve --host 127.0.0.1 --port "$PORT" >"$server_log" 2>&1 &
server_pid=$!

curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(
  printf \
    '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' \
    "$PORT"
)"

set +e
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
) >"$agent_stream" 2>"$agent_stderr"
agent_status=$?
set -e

if [[ "$agent_status" -ne 0 ]]; then
  echo "Agent CLI exited with status $agent_status; see $agent_stderr" >&2
  exit "$agent_status"
fi

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$agent_stderr"

result="$work/verdict-definition.md"
if [[ ! -f "$result" ]]; then
  echo "Agent CLI did not create verdict-definition.md" >&2
  exit 1
fi
if ! printf '%s' "$EXPECTED" | cmp -s - "$result"; then
  echo "Agent CLI created unexpected verdict-definition bytes:" >&2
  diff -u <(printf '%s' "$EXPECTED") "$result" >&2 || true
  exit 1
fi

cp "$result" "$OUT/agent-authored-verdict-definition.md"
git -C "$work" diff --check
git -C "$work" status --short >"$OUT/worktree-status.txt"
echo "issue 710 Agent CLI verdict-contract leaf passed"
