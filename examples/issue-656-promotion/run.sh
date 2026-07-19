#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8790}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-656/agent-cli-run}"
PROPOSALS="$ROOT/examples/issue-656-promotion/open-proposals.lino"
TARGET='data/seed/issue-656-agent-learned.lino'
PAYLOAD='learned_rules
  id "issue_656_agent_learning"
  rule "unseen_verified_modifier"'
TASK="Create file $TARGET containing
$PAYLOAD"

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"
promotion_work="$(mktemp -d)"
external_work="$(mktemp -d)"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$promotion_work" "$external_work"
}
trap cleanup EXIT

for work in "$promotion_work" "$external_work"; do
  git -C "$work" init -q
  git -C "$work" config user.email issue-656@example.invalid
  git -C "$work" config user.name issue-656-fixture
done

# The production promotion command executes the real canonical gates, creates a
# local promotion branch, and materializes through Formal AI's agentic driver.
(cd "$ROOT" && "$BIN" improve --promote --proposals "$PROPOSALS" \
  --apply --confirm --seed-root "$promotion_work" \
  >"$OUT/promotion-run.lino" 2>"$OUT/promotion-run.log")
git -C "$promotion_work" branch --show-current >"$OUT/promotion-branch.txt"
git -C "$promotion_work" diff --no-index -- /dev/null "$TARGET" \
  >"$OUT/promotion-result.diff" || test "$?" -eq 1

# Replay the identical literal task through the external Agent CLI against the
# Formal AI OpenAI-compatible server and preserve its raw evidence.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null
config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(cd "$external_work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
cmp "$promotion_work/$TARGET" "$external_work/$TARGET"
cp "$external_work/.formal-ai/general-change-plan.lino" "$OUT/general-change-plan.lino"
"$BIN" agent --task "$TASK" --session-json "$OUT/session.json" >/dev/null
echo "issue 656 promotion and external Agent CLI replay passed"
