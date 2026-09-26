#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8792}"
OUT="${OUT:-$REPO_ROOT/docs/case-studies/issue-922/agent-cli-run}"
PROPOSALS="$ROOT/examples/issue-922-method-learning/open-proposals.lino"
TARGET='data/seed/learned-methods.lino'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --bin formal-ai" >&2; exit 2; }
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
  git -C "$work" config user.email issue-922@example.invalid
  git -C "$work" config user.name issue-922-fixture
done
mkdir -p "$promotion_work/data/seed"
touch "$promotion_work/$TARGET"
git -C "$promotion_work" add "$TARGET"
git -C "$promotion_work" commit -qm 'seed promotion fixture'

# Production confirmation path: fresh canonical gates, clean review branch,
# and Formal AI's exact-byte agentic materializer.
(cd "$REPO_ROOT" && "$BIN" improve --promote --proposals "$PROPOSALS" \
  --apply --confirm --seed-root "$promotion_work" \
  >"$OUT/promotion-run.lino" 2>"$OUT/promotion-run.log")
git -C "$promotion_work" branch --show-current >"$OUT/promotion-branch.txt"
git -C "$promotion_work" diff >"$OUT/promotion-result.diff"
cmp "$promotion_work/$TARGET" "$REPO_ROOT/$TARGET"

# Independent external Agent CLI replay through Formal AI's live API. The
# literal task is derived from the promoted bytes; cmp below is the provenance
# assertion, not an exit-code-only smoke test.
desired="$(<"$promotion_work/$TARGET")"
task="Create file $TARGET containing
$desired"
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$external_work/memory.lino" FORMAL_AI_DREAMING=0 "$BIN" serve \
  --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null
config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
# --no-summarize-session and --compaction-models "(same)" keep every model
# call on the formal-ai provider under test: the CLI's default session-summary
# publish at exit routes through a hosted provider and rejects outside its
# client (an UnhandledRejection that fails the run even though the write
# succeeded). Same flags as the canonical agent_cli_e2e harness.
(cd "$external_work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin \
  --no-summarize-session --compaction-models "(same)" --prompt "$task" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")
"$REPO_ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
cmp "$promotion_work/$TARGET" "$external_work/$TARGET"
cp "$external_work/.formal-ai/general-change-plan.lino" "$OUT/general-change-plan.lino"
"$BIN" agent --task "$task" --session-json "$OUT/session.json" >/dev/null
"$AGENT" --version >"$OUT/agent-version.txt"
echo "issue 922 confirmed promotion and external Agent CLI replay passed"
