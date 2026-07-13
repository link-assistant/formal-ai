#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8786}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-651/self-coding-run}"
TASK='Create file self-coding-result.txt containing self-coding=passed'
if [[ "${1:-}" == "--live" ]]; then
  [[ -n "${2:-}" ]] || { echo "usage: $0 --live ISSUE_URL" >&2; exit 2; }
  exec solve "$2" --tool agent --model formal-ai --verbose
fi
command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"
work="$(mktemp -d)"
server="$OUT/formal-ai.log"
events="$OUT/agent-stream.jsonl"
raw_events="$OUT/agent-stream.raw.log"
cleanup() { kill "${server_pid:-}" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
cp "$ROOT/examples/self-coding/issue.md" "$work/ISSUE.md"
git -C "$work" add ISSUE.md
git -C "$work" commit -qm fixture
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$server" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null
config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$PORT")"
(cd "$work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" >"$raw_events")
grep '^{' "$raw_events" >"$events"
rm "$raw_events"
test "$(cat "$work/self-coding-result.txt")" = 'self-coding=passed'
git -C "$work" diff --no-index -- /dev/null self-coding-result.txt \
  >"$OUT/result.diff" || test "$?" -eq 1
cp "$work/.formal-ai/general-change-plan.lino" "$OUT/general-change-plan.lino"
"$BIN" agent --task "$TASK" --session-json "$OUT/session.json" >/dev/null
printf 'solve ISSUE_URL --tool agent --model formal-ai\n' >"$OUT/hive-mind-dispatch.log"
echo "self-coding replay passed"
