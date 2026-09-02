#!/usr/bin/env bash
# Replay only the root of the Agent ladder against two already verified child
# effects. This shortens the red/green loop for failures that appear only after
# child result lines grow beyond Agent's display width.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
PORT="${PORT:-15000}"
OUT="${OUT:-$ROOT/ci-logs/issue-1069-root-focused}"
LEFT_EFFECT="${LEFT_EFFECT:-$ROOT/docs/case-studies/issue-1028/agent-tree-run/1/effect.lino}"
RIGHT_EFFECT="${RIGHT_EFFECT:-$ROOT/docs/case-studies/issue-1028/agent-tree-run/2/effect.lino}"
VERIFY_NODE="$ROOT/experiments/issue_1028_agent_cli_ladder/verify-node.sh"

[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
[[ -s "$LEFT_EFFECT" ]] || { echo "missing left child effect: $LEFT_EFFECT" >&2; exit 2; }
[[ -s "$RIGHT_EFFECT" ]] || { echo "missing right child effect: $RIGHT_EFFECT" >&2; exit 2; }
command -v "$AGENT" >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }

mkdir -p "$OUT"
work=$(mktemp -d)
server_pid=
cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill -- "-$server_pid" 2>/dev/null || kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  cp "$work/formal-ai.log" "$OUT/formal-ai.log" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

git -C "$ROOT" archive HEAD | tar -x -C "$work"
git -C "$work" init -q
git -C "$work" config user.email agent-ladder@example.invalid
git -C "$work" config user.name agent-ladder
git -C "$work" add .
git -C "$work" commit -qm ladder-fixture
mkdir -p "$work/.agent-ladder/verified-children"
cp "$LEFT_EFFECT" "$work/.agent-ladder/verified-children/node-1.lino"
cp "$RIGHT_EFFECT" "$work/.agent-ladder/verified-children/node-2.lino"
git -C "$work" add .agent-ladder/verified-children
git -C "$work" commit -qm ladder-verified-child-effects

setsid env FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
  FORMAL_AI_MEMORY_PATH="$work/.git/formal-ai-memory/memory.lino" \
  FORMAL_AI_DREAMING=0 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$PORT" \
  >"$work/formal-ai.log" 2>&1 &
server_pid=$!
curl -fsS --retry 30 --retry-delay 1 --retry-connrefused \
  "http://127.0.0.1:$PORT/health" >/dev/null

config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$PORT")"
prompt="$(printf '%s\n\n%s\n' \
  'Verify Formal AI supports recursive binary task decomposition from atomic leaves through the complete 32-leaf level.' \
  "This is recursive binary-tree node R at depth 0. Solve only this node's task in this fresh temporary repository. Its harness-evaluated completion criterion is: new_composite_effect. Read the committed child effects in \`.agent-ladder/verified-children/node-1.lino\` and \`.agent-ladder/verified-children/node-2.lino\`. Inspect both files before writing anything. Extract each raw child value with \`sed -n \\\"s/^result=//p\\\" FILE\` or an equivalent command that returns undecorated file bytes. Treat only the single line beginning exactly \`result=\` as that child result. Do not copy tool-rendered line numbers, \`<file>\` wrappers, or any other fields. Create \`agent-ladder-effects/node-R.lino\` with these exact field lines: \`node_path=R\`, \`node_depth=0\`, \`node_kind=composite\`, \`left_child=1\`, \`right_child=2\`, \`left_result=\` followed by the exact left child \`result=\` value, \`right_result=\` followed by the exact right child \`result=\` value, and \`result=\` followed by at least four words that include both exact child result values and state how they compose. Leave supporting evidence in .agent-ladder/node-R-proof.md. The first line must be exactly node_path=R and the body must state the concrete result. The harness rejects proof without the separate Git effect. Use web research when it materially improves factual accuracy. Do not claim success without evidence.")"

(
  cd "$work"
  FORMAL_AI_API_KEY=local \
  LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --no-summarize-session --compaction-model same \
      --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin \
      --prompt "$prompt"
) >"$OUT/agent-stream.jsonl" 2>"$OUT/agent-stderr.log"

proof="$work/.agent-ladder/node-R-proof.md"
effect="$work/agent-ladder-effects/node-R.lino"
verdict=$("$VERIFY_NODE" "$work" "$proof" R 0 1 2 '' '')
cp "$proof" "$OUT/proof.md"
cp "$effect" "$OUT/effect.lino"
printf 'verdict=%s\n' "$verdict"
sha256sum "$effect"
