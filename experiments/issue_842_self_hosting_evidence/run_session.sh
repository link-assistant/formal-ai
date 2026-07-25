#!/usr/bin/env bash
# Drive one real Formal AI Agent-CLI session and capture it as self-hosting
# evidence (issue #842).
#
# The differential self-hosting gate (`scripts/self-hosting-metric.rs
# --check-ratchet`) rejects a branch whose merge would lower the projected
# self-hosting share of the next release. A branch of hand-authored work is
# answerable for its own delta, so the remedy the gate names is to record what
# Formal AI itself authored, with a session id and a committed transcript that
# contains it. This script produces exactly that bundle, the same way issue
# #834's `self-hosting-evidence/` was produced, so the claim is reproducible
# rather than asserted.
#
# What runs:
#   1. `formal-ai serve` in agent mode on a private, empty memory.
#   2. The real `@link-assistant/agent` CLI, pointed at that server as its only
#      model provider, driving the whole-repository source-links task. The CLI
#      writes `self-source-links.lino` through its own `write_file` tool; the
#      content is the server's, not the harness's.
#   3. `cargo run --example project_source_links_sharded`, the exhaustive
#      projection of every owned module, each one verified to round-trip
#      source -> links -> source byte-for-byte.
#
# Usage: experiments/issue_842_self_hosting_evidence/run_session.sh [out-dir]
#   out-dir defaults to docs/case-studies/issue-842/self-hosting-evidence.
#
# Environment knobs:
#   BIN    release-mode formal-ai binary (default: target/release/formal-ai)
#   PORT   server port (default: 8842)
#   AGENT  agent CLI (default: `agent` on PATH)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
PORT="${PORT:-8842}"
AGENT="${AGENT:-agent}"
OUT="${1:-$ROOT/docs/case-studies/issue-842/self-hosting-evidence}"

TASK="Translate the entire source code of our system to the links / meta language and back to source, and record the whole-repository source-to-links projection in Links Notation so we can recompile ourselves."

[ -x "$BIN" ] || { echo "!! build first: cargo build --release --bin formal-ai" >&2; exit 1; }
command -v "$AGENT" >/dev/null || { echo "!! agent CLI not on PATH" >&2; exit 1; }

WORKDIR="$(mktemp -d)"
LOG="$WORKDIR/formal-ai.log"
mkdir -p "$OUT"

# The CLI reads its provider config from opencode.json in the working
# directory. `@ai-sdk/openai-compatible` is the generic adapter; it POSTs to
# <baseURL>/chat/completions and reads back the SSE the server emits.
cat > "$WORKDIR/opencode.json" <<EOF
{
  "provider": {
    "formalai": {
      "name": "Formal AI",
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": "http://127.0.0.1:$PORT/api/openai/v1",
        "apiKey": "local"
      },
      "models": { "formal-ai": { "name": "Formal AI" } }
    }
  },
  "model": "formalai/formal-ai"
}
EOF

# FORMAL_AI_AGENT_MODE=1 opens the permission gate for tool-call execution.
# FORMAL_AI_TRACE_REQUESTS=1 puts the request trace in the server log, so the
# committed evidence shows the round trips, not just their result. A private
# memory file with dreaming off keeps the session independent of whatever
# earlier local runs recorded (issue #828).
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$WORKDIR/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" > "$LOG" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null || true; rm -rf "$WORKDIR"' EXIT

# curl's own retry loop is the backoff; no foreground sleep.
curl -sS --retry 30 --retry-delay 1 --retry-connrefused --max-time 40 \
  "http://127.0.0.1:$PORT/health" >/dev/null \
  || { echo "!! server never came up on $PORT" >&2; tail -40 "$LOG" >&2; exit 1; }
echo "== server up on $PORT =="

# The external CLI is non-deterministic and occasionally exits 0 after the
# first turn without walking the recipe, so retry until it actually writes the
# document (experiments/agent_cli_e2e/run_agent_cli.sh documents the same
# quirk). The server side is deterministic; only the harness stalls.
cd "$WORKDIR"
for attempt in 1 2 3 4 5; do
  echo "== agent attempt $attempt/5 =="
  timeout 300 "$AGENT" \
    --model formalai/formal-ai \
    --permission-mode auto \
    --output-format stream-json \
    --compact-json \
    --disable-stdin \
    --prompt "$TASK" \
    > "$WORKDIR/agent-stream.jsonl" 2>&1 || true
  [ -f "$WORKDIR/self-source-links.lino" ] && break
  echo "== attempt $attempt wrote no self-source-links.lino; retrying =="
done

[ -f "$WORKDIR/self-source-links.lino" ] \
  || { echo "!! the session never wrote self-source-links.lino" >&2; tail -40 "$WORKDIR/agent-stream.jsonl" >&2; exit 1; }

# The session id has to be readable from the committed transcript: that is what
# the `Formal-AI-Evidence` gate checks the `Formal-AI-Session` trailer against.
SESSION="$(grep -o 'ses_[A-Za-z0-9]\{20,\}' "$WORKDIR/agent-stream.jsonl" | head -1)"
[ -n "$SESSION" ] || { echo "!! no session id in the transcript" >&2; exit 1; }

cp "$WORKDIR/agent-stream.jsonl" "$WORKDIR/self-source-links.lino" "$LOG" "$OUT/"

# The recipe verifies a representative slice inline to stay responsive. The
# exhaustive projection -- every owned module, each round-tripped byte-for-byte
# -- is the same library invariant, driven here so the evidence covers the
# whole repository rather than the slice.
cd "$ROOT"
cargo run --release --example project_source_links_sharded -- "$OUT" \
  > "$OUT/whole-repository-projection.summary.log" 2>&1

echo "== session $SESSION =="
echo "== evidence in $OUT =="
ls -la "$OUT"
echo
echo "Commit the bundle with BOTH trailers in one paragraph (no blank line"
echo "between them, or git reports only the last -- see issue #796):"
echo
echo "  Formal-AI-Session: $SESSION"
echo "  Formal-AI-Evidence: docs/case-studies/issue-842/self-hosting-evidence/agent-stream.jsonl"
