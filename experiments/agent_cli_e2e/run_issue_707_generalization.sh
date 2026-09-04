#!/usr/bin/env bash
# Issue #707 acceptance, generalization half: drive the twelve *held-out*
# requests from data/benchmarks/computer-use-generalization.lino twice through
# the real @link-assistant/agent CLI and Formal AI's HTTP + MCP surfaces.
#
# None of these prompts is in the recorded corpus, so every plan the external
# client executes was synthesized from the auto-learned schemas. The local run
# of each prompt is the reference: the Agent CLI must reach the same plan id and
# the same primitive sequence over the wire, in both the record and replay
# phases.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8909}"
AGENT_TIMEOUT_SECONDS="${AGENT_TIMEOUT_SECONDS:-120}"

# Issue #1069: the run owns a deadline, not only each session. Twenty sessions
# entitled to AGENT_TIMEOUT_SECONDS each ask for 2400s, and no step budget in
# the agent-CLI job can hold that -- run 33880485514 spent a whole 10-minute
# step on the record phase alone and was killed by the runner, which names the
# step and not the scenario that ran long. `scripts/run-with-budget-warning.sh`
# enforces the same budget one level up; the reserve leaves the verifier room
# after the loop, so this script fails first and says where the time went.
TEST_BUDGET_SECONDS="${TEST_BUDGET_SECONDS:-600}"
VERIFY_RESERVE_SECONDS="${VERIFY_RESERVE_SECONDS:-60}"
LOOP_DEADLINE_SECONDS=$((TEST_BUDGET_SECONDS - VERIFY_RESERVE_SECONDS))
[[ "$LOOP_DEADLINE_SECONDS" -gt 0 ]] || {
  echo "TEST_BUDGET_SECONDS must leave room for VERIFY_RESERVE_SECONDS" >&2
  exit 2
}
SUITE="$ROOT/data/benchmarks/computer-use-generalization.lino"
EVIDENCE_DIR="${EVIDENCE_DIR:-$ROOT/docs/case-studies/issue-707/agent-cli-evidence/generalization}"
WORKDIR="$(mktemp -d)"
SERVER_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  echo "::error title=issue #707 held-out computer-use generalization::$1" >&2
  echo "!! $1" >&2
  if [[ -n "$SERVER_PID" ]]; then
    tail -120 "$WORKDIR/${CURRENT_PHASE:-record}/server.log" >&2 2>/dev/null || true
  fi
  exit 1
}

command -v "$AGENT" >/dev/null
command -v node >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --bin formal-ai" >&2
  exit 2
}

mapfile -t CASE_IDS < <(sed -n 's/^  case //p' "$SUITE")
mapfile -t CASE_PROMPTS < <(sed -n 's/^    prompt en "\(.*\)"$/\1/p' "$SUITE")
[[ "${#CASE_IDS[@]}" -ge 12 ]] || fail "expected at least twelve held-out cases"
[[ "${#CASE_IDS[@]}" -eq "${#CASE_PROMPTS[@]}" ]] || fail "every case needs an English prompt"

mkdir -p "$EVIDENCE_DIR"

# The reference: synthesize and execute each held-out prompt locally, and record
# the plan the verifier will hold the external client to.
echo "== deriving synthesized reference plans =="
: >"$WORKDIR/expected.jsonl"
for index in "${!CASE_IDS[@]}"; do
  case_id="${CASE_IDS[$index]}"
  prompt="${CASE_PROMPTS[$index]}"
  "$BIN" computer-use --agent-mode --confirm-effects --prompt "$prompt" \
    >"$WORKDIR/$case_id.local.json" 2>"$WORKDIR/$case_id.local.err" \
    || fail "local synthesis failed for $case_id"
  node -e '
    const fs = require("node:fs");
    const [file, caseId, prompt] = process.argv.slice(1);
    const outcome = JSON.parse(fs.readFileSync(file, "utf8")).outcome;
    if (!outcome.verified) throw new Error(`${caseId} did not verify locally`);
    if (!outcome.plan.id.startsWith("synthesized-")) {
      throw new Error(`${caseId} was recalled, not synthesized: ${outcome.plan.id}`);
    }
    process.stdout.write(JSON.stringify({
      case_id: caseId,
      prompt,
      plan_id: outcome.plan.id,
      steps: outcome.plan.steps.map((step) => step.primitive),
    }) + "\n");
  ' "$WORKDIR/$case_id.local.json" "$case_id" "$prompt" >>"$WORKDIR/expected.jsonl" \
    || fail "local reference rejected for $case_id"
done
cp "$WORKDIR/expected.jsonl" "$EVIDENCE_DIR/expected.jsonl"

for phase in record replay; do
  mkdir -p "$EVIDENCE_DIR/$phase" "$WORKDIR/$phase"
  CURRENT_PHASE="$phase"
  cat >"$WORKDIR/$phase/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "provider": {
    "formal-ai": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Formal AI",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/v1",
        "apiKey": "local"
      },
      "models": {
        "formal-ai": { "name": "Formal AI verified computer use" }
      }
    }
  },
  "mcp": {
    "formal_ai": {
      "type": "remote",
      "url": "http://127.0.0.1:$PORT/mcp",
      "enabled": true,
      "tool_call_timeout": 120000
    }
  },
  "mcp_defaults": {
    "tool_call_timeout": 120000,
    "max_tool_call_timeout": 600000
  }
}
EOF

  : >"$EVIDENCE_DIR/$phase/audit.jsonl"
  FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=0 \
    FORMAL_AI_COMPUTER_USE_AUDIT_PATH="$EVIDENCE_DIR/$phase/audit.jsonl" \
    FORMAL_AI_MEMORY_PATH="$WORKDIR/$phase/memory.lino" FORMAL_AI_DREAMING=0 \
    "$BIN" serve --host 127.0.0.1 --port "$PORT" \
    >"$WORKDIR/$phase/server.log" 2>&1 &
  SERVER_PID=$!
  curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
    "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
    || fail "$phase server never became healthy"

  for index in "${!CASE_IDS[@]}"; do
    case_id="${CASE_IDS[$index]}"
    prompt="${CASE_PROMPTS[$index]}"
    log="$EVIDENCE_DIR/$phase/$case_id.jsonl"
    remaining=$((LOOP_DEADLINE_SECONDS - SECONDS))
    if [[ "$remaining" -le 0 ]]; then
      fail "the ${TEST_BUDGET_SECONDS}s run budget was spent before $phase/$case_id started"
    fi
    session_seconds="$AGENT_TIMEOUT_SECONDS"
    [[ "$session_seconds" -le "$remaining" ]] || session_seconds="$remaining"
    echo "== $phase $((index + 1))/${#CASE_IDS[@]}: $case_id (t+${SECONDS}s of ${LOOP_DEADLINE_SECONDS}s) =="
    session_status=0
    (
      cd "$WORKDIR/$phase"
      timeout "$session_seconds" "$AGENT" \
        --prompt "$prompt" \
        --mcp-default-tool-call-timeout 120000 \
        --mcp-max-tool-call-timeout 600000 \
        --disable-stdin \
        --model formal-ai/formal-ai \
        --no-summarize-session \
        --compaction-model same \
        --output-format stream-json \
        --compact-json
    ) >"$log" 2>&1 || session_status=$?
    if [[ "$session_status" -eq 124 ]]; then
      if [[ "$session_seconds" -lt "$AGENT_TIMEOUT_SECONDS" ]]; then
        fail "the ${TEST_BUDGET_SECONDS}s run budget expired inside $phase/$case_id, which \
started with ${session_seconds}s of its ${AGENT_TIMEOUT_SECONDS}s left"
      fi
      fail "$phase/$case_id outlasted its ${session_seconds}s session deadline"
    fi
    [[ "$session_status" -eq 0 ]] ||
      fail "Agent CLI failed for $phase/$case_id (exit ${session_status})"
    grep -q '"session_id":"ses_' "$log" \
      || fail "Agent CLI did not preserve a session id for $phase/$case_id"
    grep -q "computer_use_complete" "$log" \
      || fail "Formal AI did not complete $phase/$case_id"
  done

  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""
done

node "$ROOT/experiments/agent_cli_e2e/verify_issue_707_generalization.mjs" \
  "$EVIDENCE_DIR/expected.jsonl" "$EVIDENCE_DIR"

echo "== issue #707 held-out generalization passed through the real Agent CLI =="
echo "evidence: $EVIDENCE_DIR"
