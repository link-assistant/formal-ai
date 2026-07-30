#!/usr/bin/env bash
# Real Formal-AI-server -> Agent-CLI proof for issue #707's first red test.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8807}"
AGENT_TIMEOUT_SECONDS="${AGENT_TIMEOUT_SECONDS:-90}"
EVIDENCE_DIR="${EVIDENCE_DIR:-$ROOT/docs/case-studies/issue-707/agent-cli-evidence/self-authorship}"
WORKDIR="$(mktemp -d)"
SERVER_LOG="$EVIDENCE_DIR/formal-ai.log"
AGENT_LOG="$EVIDENCE_DIR/agent-stream.jsonl"
GENERATED="$EVIDENCE_DIR/issue_707_seed_taxonomy.rs"
CANONICAL="$ROOT/tests/issue_707_seed_taxonomy.rs"
SERVER_PID=""
TASK='Create file issue_707_seed_taxonomy.rs containing
//! Agent CLI-authored red regression for GitHub issue #707.

use std::fs;

const REQUIRED: [&str; 12] = [
    "fs.read",
    "fs.write",
    "fs.list",
    "fs.move",
    "shell.run",
    "http.fetch",
    "http.post",
    "dom.query",
    "dom.extract",
    "archive.pack",
    "archive.unpack",
    "process.status",
];

#[test]
fn computer_use_primitive_taxonomy_is_seeded() {
    let seed = fs::read_to_string("data/seed/tools.lino").expect("tool registry");
    for primitive in REQUIRED {
        assert!(
            seed.contains(&format!("name {primitive}")),
            "missing primitive {primitive}"
        );
    }
}'
EXPECTED='//! Agent CLI-authored red regression for GitHub issue #707.

use std::fs;

const REQUIRED: [&str; 12] = [
    "fs.read",
    "fs.write",
    "fs.list",
    "fs.move",
    "shell.run",
    "http.fetch",
    "http.post",
    "dom.query",
    "dom.extract",
    "archive.pack",
    "archive.unpack",
    "process.status",
];

#[test]
fn computer_use_primitive_taxonomy_is_seeded() {
    let seed = fs::read_to_string("data/seed/tools.lino").expect("tool registry");
    for primitive in REQUIRED {
        assert!(
            seed.contains(&format!("name {primitive}")),
            "missing primitive {primitive}"
        );
    }
}'

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT

fail() {
  echo "!! $1" >&2
  tail -100 "$AGENT_LOG" >&2 2>/dev/null || true
  tail -160 "$SERVER_LOG" >&2 2>/dev/null || true
  exit 1
}

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || {
  echo "build first: cargo build --bin formal-ai" >&2
  exit 2
}

mkdir -p "$EVIDENCE_DIR"
cat >"$WORKDIR/opencode.json" <<EOF
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
        "formal-ai": { "name": "Formal AI Symbolic Production" }
      }
    }
  }
}
EOF

FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 \
  || fail "Formal AI server never came up"

(
  cd "$WORKDIR"
  timeout "$AGENT_TIMEOUT_SECONDS" "$AGENT" \
    --prompt "$TASK" \
    --disable-stdin \
    --model formal-ai/formal-ai \
    --no-summarize-session \
    --compaction-model same \
    --output-format stream-json \
    --compact-json \
    --verbose
) >"$AGENT_LOG" 2>&1 || fail "Agent CLI did not complete"

[[ -f "$WORKDIR/issue_707_seed_taxonomy.rs" ]] \
  || fail "Agent CLI did not author the regression"
printf '%s' "$EXPECTED" | cmp -s - "$WORKDIR/issue_707_seed_taxonomy.rs" \
  || fail "Agent CLI artifact differs from the reviewed leaf"
cp "$WORKDIR/issue_707_seed_taxonomy.rs" "$GENERATED"
cp "$WORKDIR/issue_707_seed_taxonomy.rs" "$CANONICAL"
if [[ -f "$WORKDIR/.formal-ai/general-change-plan.lino" ]]; then
  cp "$WORKDIR/.formal-ai/general-change-plan.lino" \
    "$EVIDENCE_DIR/general-change-plan.lino"
fi

grep -q '"session_id":"ses_' "$AGENT_LOG" \
  || fail "Agent CLI stream did not preserve its session id"
grep -q 'agentic_outcome: planned Final' "$SERVER_LOG" \
  || fail "Formal AI did not finish the self-authoring recipe"
grep -q 'agentic_outcome: planned ToolCalls.*write' "$SERVER_LOG" \
  || fail "Formal AI did not drive a write step"
grep -q 'agentic_outcome: planned ToolCalls.*bash' "$SERVER_LOG" \
  || fail "Formal AI did not verify the authored bytes"

set +e
cargo test --manifest-path "$ROOT/Cargo.toml" --test issue_707_seed_taxonomy \
  >"$EVIDENCE_DIR/red-test.log" 2>&1
test_status=$?
set -e
[[ "$test_status" -ne 0 ]] || fail "taxonomy regression unexpectedly passed before implementation"
grep -q 'missing primitive fs.read' "$EVIDENCE_DIR/red-test.log" \
  || fail "red test did not fail for the missing taxonomy"

echo "== issue #707 self-authored red test OK =="
grep -m1 -o '"session_id":"ses_[^"]*"' "$AGENT_LOG"
