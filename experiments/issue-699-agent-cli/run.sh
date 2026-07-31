#!/usr/bin/env bash
set -euo pipefail

# Issue #699 — Formal AI authors the machine-pinned verification record for
# the first specialized-handler migration batch through the real Agent CLI.

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8899}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-699/agent-cli-evidence}"
TASK='Create file handler-migration-batch-report.lino containing handler_migration_batch:number_constraints status "migrated" recognition "seed_roles" native_primitive "interval_reasoning" held_out_languages "en,ru,hi,zh".'
EXPECTED='handler_migration_batch:number_constraints status "migrated" recognition "seed_roles" native_primitive "interval_reasoning" held_out_languages "en,ru,hi,zh".'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}

mkdir -p "$OUT"
work="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue-699.XXXXXX")"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
printf '%s\n' '# Issue 699 self-coding fixture' >"$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture
printf '%s\n' "$TASK" >"$OUT/task.txt"

FORMAL_AI_AGENT_MODE=1 \
FORMAL_AI_TRACE_REQUESTS=1 \
FORMAL_AI_MEMORY_PATH="$work/memory.lino" \
FORMAL_AI_DREAMING=0 \
"$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
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

result="$work/handler-migration-batch-report.lino"
if [[ ! -f "$result" ]]; then
  echo "Agent CLI did not create handler-migration-batch-report.lino" >&2
  exit 1
fi
if ! printf '%s' "$EXPECTED" | cmp -s - "$result"; then
  echo "Agent CLI created unexpected verification bytes:" >&2
  diff -u <(printf '%s' "$EXPECTED") "$result" >&2 || true
  exit 1
fi

cp "$result" "$OUT/handler-migration-batch-report.lino"
git -C "$work" status --short >"$OUT/worktree-status.txt"
echo "issue #699 Agent CLI verification leaf passed"
