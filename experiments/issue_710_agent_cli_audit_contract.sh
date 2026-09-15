#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8711}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-710/agent-cli-evidence/audit-contract}"
CASE_STUDY="$ROOT/docs/case-studies/issue-710/README.md"

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --release --bin formal-ai" >&2
  exit 2
}

mkdir -p "$OUT"
work="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-issue-710-audit.XXXXXX")"
expected="$work/expected-audit-contract.lino"
server_log="$OUT/formal-ai-server.log"
agent_stream="$OUT/agent-stream.raw.log"
agent_stderr="$OUT/agent-stderr.log"
cleanup() {
  kill "${server_pid:-}" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

awk -F '|' '
  function clean(value) {
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
    gsub(/"/, "\\\"", value)
    return value
  }
  BEGIN { print "issue_710_audit_contract" }
  /^\| [[:space:]]*[0-9]+ [|]/ {
    row_index = clean($2)
    area = clean($3)
    verdict = clean($5)
    gsub(/`/, "", verdict)
    evidence_kind = "regression"
    if (verdict == "still-broken") evidence_kind = "focused-tracker"
    if (verdict == "superseded") evidence_kind = "superseding-work"
    count += 1
    printf "  requirement\n"
    printf "    index \"%s\"\n", row_index
    printf "    identifier \"R710-%02d\"\n", row_index
    printf "    area \"%s\"\n", area
    printf "    verdict \"%s\"\n", verdict
    printf "    evidence_kind \"%s\"\n", evidence_kind
    printf "    evidence_ref \"case-study-R710-%02d\"\n", row_index
    printf "    reviewed_at \"2026-08-01\"\n"
  }
  END {
    if (count != 32) {
      print "expected 32 issue-710 rows, got " count > "/dev/stderr"
      exit 42
    }
  }
' "$CASE_STUDY" >"$expected"
perl -pi -e 'chomp if eof' "$expected"

task="Create file issue-710-audit-contract.lino containing $(cat "$expected")"

git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
printf '%s\n' '# Issue 710 audit-contract fixture' >"$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture
printf '%s\n' "$task" >"$OUT/task.log"

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
    --prompt "$task"
) >"$agent_stream" 2>"$agent_stderr"
agent_status=$?
set -e

if [[ "$agent_status" -ne 0 ]]; then
  echo "Agent CLI exited with status $agent_status; see $agent_stderr" >&2
  exit "$agent_status"
fi

"$ROOT/scripts/classify-agent-cli-stderr.sh" "$agent_stderr"

result="$work/issue-710-audit-contract.lino"
if [[ ! -f "$result" ]]; then
  echo "Agent CLI did not create issue-710-audit-contract.lino" >&2
  exit 1
fi
if ! cmp -s "$expected" "$result"; then
  echo "Agent CLI created unexpected audit-contract bytes:" >&2
  diff -u "$expected" "$result" >&2 || true
  exit 1
fi

cp "$result" "$OUT/agent-authored-audit-contract.lino"
git -C "$work" diff --check
git -C "$work" status --short >"$OUT/worktree-status.log"
echo "issue 710 Agent CLI 32-row audit-contract leaf passed"
