#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-1028/agent-tree-run}"
TREE_DEPTH="${TREE_DEPTH:-5}"
NODE_FILTER="${NODE_FILTER:-}"
BASE_PORT="${BASE_PORT:-8870}"

[[ -x "$BIN" ]] || { echo "build first: cargo build --release --bin formal-ai" >&2; exit 2; }
command -v "$AGENT" >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }
command -v git >/dev/null || { echo "git is required" >&2; exit 2; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }

case "$TREE_DEPTH" in
  0|1|2|3|4|5|all) ;;
  *) echo "TREE_DEPTH must be 0, 1, 2, 3, 4, 5, or all" >&2; exit 2 ;;
esac
if [[ -n "$NODE_FILTER" && ! "$NODE_FILTER" =~ ^(R|[12](\.[12]){0,4})$ ]]; then
  echo "NODE_FILTER must be R or a binary path such as 2.1.2.2.1" >&2
  exit 2
fi

mkdir -p "$OUT"
NODES="$OUT/tree.tsv"
RUN_LOG="$OUT/run.log"
: > "$NODES"
: > "$RUN_LOG"

cat > "$OUT/leaves.tsv" <<'EOF'
L01	Inspect the existing task-decomposition data model and identify where a node stores its children.
L02	Inspect the existing task-decomposition recursion and record how depth limits are represented.
L03	Inspect the existing atomicity check and record the observable completion contract for leaves.
L04	Inspect the existing Links Notation rendering and record how child relationships are serialized.
L05	Inspect the existing recursive execution adapter and record how a decomposition tree is executed.
L06	Inspect the existing task-strategy ledger and record how approved decomposition strategies are selected.
L07	Write a minimal example of a two-child task decomposition with independently checkable leaves.
L08	Verify that a leaf without an observable completion contract is never treated as independently checkable.
L09	Inspect the binary decomposition invariant and explain the exactly-two-children requirement.
L10	Verify the invariant explicitly names the supported power-of-two levels through 32.
L11	Add or verify regression coverage for a two-node decomposition at depth one.
L12	Add or verify regression coverage for a four-node decomposition at depth two.
L13	Add or verify regression coverage for an eight-node decomposition at depth three.
L14	Add or verify regression coverage for a sixteen-node decomposition at depth four.
L15	Add or verify regression coverage for a thirty-two-node decomposition at depth five.
L16	Verify every tested internal node has exactly two children and never three or more.
L17	Verify every tested leaf is atomic and independently checkable.
L18	Verify every tested node has a stable id and a unique dotted path.
L19	Verify child paths follow the binary 1/2 convention at every depth.
L20	Verify the node count of a complete depth-five tree is exactly 63 including the root.
L21	Inspect the Agent-CLI ladder workflow and verify depth selection supports 0 through 5 and all.
L22	Verify a single node can be selected by dotted binary path for focused debugging.
L23	Verify the ladder can execute the 32 smallest leaves before moving to larger composite nodes.
L24	Verify the ladder order for all mode is 32, 16, 8, 4, 2, then the root.
L25	Verify every selected node runs in a fresh temporary repository copy.
L26	Verify every selected node uses the real Agent CLI against the real Formal AI server.
L27	Verify every selected node requires an observable proof file with its exact node path.
L28	Inspect the committed binary-tree case-study and verify it describes a tree rather than a flat list.
L29	Verify the case-study lists exactly 32 distinct atomic leaf formulations.
L30	Verify the case-study path structure contains every binary path from depth one through five.
L31	Use a differently worded decomposition request to check that the capability is not phrase-specific.
L32	Produce a final evidence note containing the selected tree level, node outcomes, test results, and session id.
EOF

python3 - "$OUT/leaves.tsv" "$NODES" <<'PY'
import sys
from pathlib import Path
leaves = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    leaf, text = line.split('\t', 1)
    leaves[int(leaf[1:])] = text

def child(path, branch):
    return path + ("." if path else "") + str(branch)

def leaf_index(path):
    bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
    return int(bits, 2) + 1

def emit(path, depth, out):
    if depth == 0:
        text = 'Verify Formal AI supports recursive binary task decomposition from atomic leaves through the complete 32-leaf level.'
        criterion = 'all_children_pass'
        node_id = 'R'
    elif depth == 5:
        i = leaf_index(path)
        node_id = path
        text = f'Atomic task L{i:02d}: {leaves[i]}'
        criterion = 'observable evidence exists'
    else:
        node_id = path
        bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
        prefix = int(bits, 2)
        span = 2 ** (5 - depth)
        start = prefix * span + 1
        end = (prefix + 1) * span
        text = f'Complete recursive decomposition node {path}, covering atomic tasks L{start:02d}–L{end:02d}; both child nodes must produce independently checkable evidence.'
        criterion = 'all_children_pass'
    left = child(path, 1) if depth < 5 else ''
    right = child(path, 2) if depth < 5 else ''
    out.append((node_id, depth, text, criterion, left, right))
    if depth < 5:
        emit(child(path,1), depth+1, out)
        emit(child(path,2), depth+1, out)

rows=[]
emit('',0,rows)
# `depth` is an int, and str.join refuses a non-str item, so joining the row
# straight raised TypeError before a single node was ever selected. Render
# every field before joining rather than trusting the tuple to be all strings.
Path(sys.argv[2]).write_text('\n'.join('\t'.join(map(str, r)) for r in rows)+'\n')
PY

python3 - "$NODES" "$TREE_DEPTH" "$NODE_FILTER" > "$OUT/selected.tsv" <<'PY'
import sys
from pathlib import Path
rows=[]
for line in Path(sys.argv[1]).read_text().splitlines():
    node, depth, text, criterion, left, right = line.split('\t', 5)
    rows.append((node,int(depth),text,criterion,left,right))
mode=sys.argv[2]
filt=sys.argv[3]
levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
for level in levels:
    for row in rows:
        node, depth, *_ = row
        if depth == level and (not filt or node == filt):
            print('\t'.join(map(str,row)))
PY

selected_count=$(wc -l < "$OUT/selected.tsv" | tr -d ' ')
expected=1
if [[ "$TREE_DEPTH" = all ]]; then
  expected=63
elif [[ -n "$NODE_FILTER" ]]; then
  expected=1
else
  expected=$((1 << TREE_DEPTH))
fi
[[ "$selected_count" -eq "$expected" ]] || { echo "expected $expected selected nodes, got $selected_count" >&2; exit 1; }

run_one() {
  local id depth prompt criterion work session_dir server_pid port status proof config node_number full_prompt
  IFS=$'\t' read -r id depth prompt criterion _left _right <<< "$1"
  session_dir="$OUT/$id"
  work=$(mktemp -d)
  mkdir -p "$session_dir"
  node_number=$(python3 - "$id" <<'PY'
import sys
node=sys.argv[1]
if node == 'R':
    print(0)
else:
    bits=''.join('0' if x == '1' else '1' for x in node.split('.'))
    print(int(bits, 2) + 1)
PY
)
  port=$((BASE_PORT + node_number))

  cleanup_one() {
    if [[ -n "${server_pid:-}" ]]; then
      kill -- "-${server_pid}" 2>/dev/null || kill "$server_pid" 2>/dev/null || true
      wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$work"
  }
  trap cleanup_one RETURN

  git -C "$ROOT" archive HEAD | tar -x -C "$work"
  git -C "$work" init -q
  git -C "$work" config user.email agent-ladder@example.invalid
  git -C "$work" config user.name agent-ladder
  git -C "$work" add .
  git -C "$work" commit -qm ladder-fixture
  mkdir -p "$work/.agent-ladder"

  setsid env FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.agent-ladder/memory.lino" \
    FORMAL_AI_DREAMING=0 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$port" \
    >"$session_dir/formal-ai.log" 2>&1 &
  server_pid=$!

  if ! curl -fsS --retry 30 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$port/health" >/dev/null; then
    printf '%s\tFAIL\tformal_ai_server_start\n' "$id" >> "$RUN_LOG"
    tail -100 "$session_dir/formal-ai.log" >&2 || true
    return 1
  fi

  config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$port")"

  # Built with printf, not interpolated into a double-quoted string: bash does
  # not expand \n there, so the node instructions used to reach the agent as one
  # line with two literal backslash-n in the middle of it.
  printf -v full_prompt '%s\n\nThis is recursive binary-tree node %s at depth %s. Solve only this node'"'"'s task in this fresh temporary repository. Its completion criterion is: %s. Leave observable evidence in .agent-ladder/node-%s-proof.md. The first line must be exactly node_path=%s. Use web research when it materially improves factual accuracy. Do not claim success without evidence.\n' \
    "$prompt" "$id" "$depth" "$criterion" "$id" "$id"

  set +e
  (cd "$work" && \
    FORMAL_AI_API_KEY=local \
    LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin \
      --prompt "$full_prompt") \
      >"$session_dir/agent-stream.jsonl" 2>"$session_dir/agent-stderr.log"
  status=$?
  set -e
  if [[ "$status" -ne 0 ]]; then
    printf '%s\tFAIL\tagent_exit_%s\n' "$id" "$status" >> "$RUN_LOG"
    return 1
  fi

  proof="$work/.agent-ladder/node-${id}-proof.md"
  if [[ ! -s "$proof" ]]; then
    printf '%s\tFAIL\tmissing_proof\n' "$id" >> "$RUN_LOG"
    return 1
  fi
  if ! grep -q "^node_path=$id$" "$proof"; then
    printf '%s\tFAIL\tbad_proof_marker\n' "$id" >> "$RUN_LOG"
    return 1
  fi

  cp "$proof" "$session_dir/proof.md"
  printf '%s\tPASS\tdepth=%s\n' "$id" "$depth" >> "$RUN_LOG"
}

failed=0
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  node=$(printf '%s\n' "$line" | cut -f1)
  echo "=== $node ===" | tee -a "$RUN_LOG"
  if run_one "$line"; then :; else failed=1; break; fi
done < "$OUT/selected.tsv"

cat > "$OUT/README.md" <<EOF
# Recursive Formal AI decomposition tree run

- requested depth: $TREE_DEPTH
- node filter: ${NODE_FILTER:-none}
- selected nodes: $selected_count
- failures: $failed

The canonical decomposition is a complete binary tree: depth 0 has 1 node,
depth 1 has 2, depth 2 has 4, depth 3 has 8, depth 4 has 16, and depth 5 has 32.
Each selected node runs in a fresh temporary repository copy against the real
\`@link-assistant/agent\` CLI and a local \`formal-ai serve --agent-mode\`.

The \`all\` mode verifies the smallest atomic tasks first (32 leaves), then
16, 8, 4, 2, and finally the root, stopping on the first real failure so the
underlying capability can be repaired before larger composite tasks are tested.
EOF

exit "$failed"
