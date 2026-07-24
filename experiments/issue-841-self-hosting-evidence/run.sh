#!/usr/bin/env bash
# Issue #841 self-hosting evidence. The reusable PTY/TUI implementation lives
# in command-stream and agent-commander, while this repository consumes those
# packages in its real agent-CLI E2E matrix. To satisfy Formal AI's development
# policy with a genuine self-authored leaf, this recipe drives the real Agent
# CLI through the branch's local Formal AI server and asks Formal AI to author
# its source-to-links projection. The projection and general change plan are
# copied byte-for-byte from that session's isolated workspace.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8841}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-841/self-hosting-evidence}"
TASK='Translate the complete source of this system into the links meta-language and back again, preserving a source-to-links projection that demonstrates the repository can describe and recompile itself.'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}
mkdir -p "$OUT"
work="$(mktemp -d)"
memory="$(mktemp -d)"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work" "$memory"
}
trap cleanup EXIT

git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
touch "$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$memory/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(
  cd "$work"
  FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --model formalai/formal-ai --permission-mode auto \
    --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
    >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log"
)
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
rm "$OUT/agent-stream.raw.log" "$OUT/agent-stderr.log"

cp "$work/self-source-links.lino" "$OUT/self-source-links.lino"
cp "$work/.formal-ai/general-change-plan.lino" "$OUT/general-change-plan.lino"
"$BIN" agent --task "$TASK" --session-json "$OUT/session.json" >/dev/null

echo "issue #841 self-hosting evidence written to $OUT"
echo "session id(s): $(grep -o 'ses_[A-Za-z0-9]*' "$OUT/agent-stream.jsonl" | sort -u | tr '\n' ' ')"
