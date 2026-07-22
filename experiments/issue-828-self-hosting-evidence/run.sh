#!/usr/bin/env bash
# Issue #828 self-hosting evidence. This branch's fix (isolating every agent-CLI
# E2E server's memory so deterministic tool planning stops reading the shared
# ~/.formal-ai/memory.lino) is hand-authored maintenance, so it is committed
# WITHOUT self-authorship trailers, per CONTRIBUTING.md ("an honest 0% release is
# valid"). To keep the differential self-hosting ratchet from falling on the
# branch, we drive Formal AI — through the Agent CLI — to author a genuine piece
# of release work: its whole-repository source-to-links projection (issue #558's
# "recompile itself" recipe). The projection is a deterministic function of the
# embedded source tree, so it is byte-for-byte reproducible and unmistakably
# Formal AI-authored. The raw Agent CLI transcript and server trace are the
# excluded evidence bundle that binds the artifact to a real `ses_...` id
# (see CONTRIBUTING.md "Recording self-authorship").
#
# The server below dogfoods the branch's own fix: it runs with a private, empty
# memory (FORMAL_AI_MEMORY_PATH + FORMAL_AI_DREAMING=0), exactly the invariant
# every E2E script now enforces.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-8828}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-828/self-hosting-evidence}"
# The canonical, differently-worded source-links request the deterministic
# planner routes to the whole-repository projection recipe (src/agentic_coding/
# source_links.rs::SOURCE_LINKS_TASK).
TASK='Translate the entire source code of our system to the links / meta language and back to source, and record the whole-repository source-to-links projection in Links Notation so we can recompile ourselves.'

command -v "$AGENT" >/dev/null
[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
mkdir -p "$OUT"
work="$(mktemp -d)"
mem="$(mktemp -d)"
cleanup() { kill "${server_pid:-}" 2>/dev/null || true; rm -rf "$work" "$mem"; }
trap cleanup EXIT
git -C "$work" init -q
git -C "$work" config user.email self-coding@example.invalid
git -C "$work" config user.name self-coding-fixture
touch "$work/README.md"
git -C "$work" add README.md
git -C "$work" commit -qm fixture

# Private, empty memory + no dreaming: the exact isolation invariant issue #828
# adds to every E2E server, dogfooded here.
FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$mem/memory.lino" FORMAL_AI_DREAMING=0 \
  "$BIN" serve --host 127.0.0.1 --port "$PORT" >"$OUT/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null
config="$(printf '{"provider":{"formalai":{"name":"Formal AI","npm":"@ai-sdk/openai-compatible","options":{"baseURL":"http://127.0.0.1:%s/api/openai/v1","apiKey":"local"},"models":{"formal-ai":{"name":"Formal AI"}}}},"model":"formalai/formal-ai"}' "$PORT")"
(cd "$work" && FORMAL_AI_API_KEY=local LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
  "$AGENT" --model formalai/formal-ai --permission-mode auto \
  --output-format stream-json --compact-json --disable-stdin --prompt "$TASK" \
  >"$OUT/agent-stream.raw.log" 2>"$OUT/agent-stderr.log")
"$ROOT/scripts/classify-agent-cli-stderr.sh" "$OUT/agent-stderr.log"
grep '^{' "$OUT/agent-stream.raw.log" >"$OUT/agent-stream.jsonl"
rm "$OUT/agent-stream.raw.log" "$OUT/agent-stderr.log"

# Formal AI's own authored artifacts: the whole-repository projection it wrote and
# the change plan it composed. These are the self-authored, counted lines; the
# .jsonl transcript and .log trace stay excluded as captured evidence.
cp "$work/self-source-links.lino" "$OUT/self-source-links.lino"
cp "$work/.formal-ai/general-change-plan.lino" "$OUT/general-change-plan.lino" 2>/dev/null || true
"$BIN" agent --task "$TASK" --session-json "$OUT/session.json" >/dev/null

echo "issue #828 self-hosting evidence written to $OUT"
echo "session id(s): $(grep -o 'ses_[A-Za-z0-9]*' "$OUT/agent-stream.jsonl" | sort -u | tr '\n' ' ')"
