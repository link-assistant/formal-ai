#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${BIN:-$ROOT/target/release/formal-ai}"
AGENT="${AGENT:-agent}"
OUT="${OUT:-$ROOT/docs/case-studies/issue-1028/agent-tree-run}"
TREE_DEPTH="${TREE_DEPTH:-5}"
NODE_FILTER="${NODE_FILTER:-}"
BASE_PORT="${BASE_PORT:-8870}"
VERIFY_NODE="$ROOT/experiments/issue_1028_agent_cli_ladder/verify-node.sh"

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

# A full depth-five run is over half an hour of real Agent CLI turns, and it
# shares `target/` with whatever else is building on the machine. A rebuild or a
# cache prune inside that window swaps -- or removes -- the binary under
# measurement, and a node that fails because its server never started is
# indistinguishable in the log from a node that failed on its merits. Copy the
# binary out once, and every node is measured against the same bytes.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp "$BIN" "$STAGE/formal-ai"
BIN="$STAGE/formal-ai"

NODES="$OUT/tree.tsv"
RUN_LOG="$OUT/run.log"
: > "$NODES"
: > "$RUN_LOG"
declare -A VERIFIED_EFFECTS=()

cat > "$OUT/leaves.tsv" <<'EOF'
L01	Inspect the existing task-decomposition data model and identify where a node stores its children.	src/task_decomposition.rs	pub children: Vec<Self>
L02	Inspect the existing task-decomposition recursion and record how depth limits are represented.	src/task_decomposition.rs	pub max_depth: u8
L03	Inspect the existing atomicity check and record the observable completion contract for leaves.	src/task_decomposition.rs	!self.completion_criterion.starts_with("unresolved_")
L04	Inspect the existing Links Notation rendering and record how child relationships are serialized.	src/task_decomposition.rs	pairs.push(("child", child.id.clone()))
L05	Inspect the existing recursive execution adapter and record how a decomposition tree is executed.	src/task_decomposition.rs	children: self.children.iter().map(Self::to_recursive_task).collect()
L06	Inspect the existing task-strategy ledger and record how approved decomposition strategies are selected.	src/task_decomposition.rs	decompose_task_with_ledger(task, max_depth, &TaskStrategyLedger::shipped())
L07	Inspect the issue-sized decomposition regression and record its required lower bound on independently checkable leaves.	tests/unit/specification/task_decomposition.rs	decomposition.leaves().len() >= 3
L08	Verify that a leaf without an observable completion contract is never treated as independently checkable.	src/task_decomposition.rs	"unresolved_single_need"
L09	Inspect the binary decomposition invariant and explain the exactly-two-children requirement.	docs/case-studies/issue-1028/task-decomposition.md	Every internal node has exactly two children
L10	Verify the invariant explicitly names the supported power-of-two levels through 32.	docs/case-studies/issue-1028/task-decomposition.md	2, 4, 8, 16, and 32 nodes respectively
L11	Verify regression coverage includes two decomposition nodes at depth one.	tests/unit/issue_1066_agent_ladder.rs	BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])
L12	Verify regression coverage includes four decomposition nodes at depth two.	tests/unit/issue_1066_agent_ladder.rs	BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])
L13	Verify regression coverage includes eight decomposition nodes at depth three.	tests/unit/issue_1066_agent_ladder.rs	BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])
L14	Verify regression coverage includes sixteen decomposition nodes at depth four.	tests/unit/issue_1066_agent_ladder.rs	BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])
L15	Verify regression coverage includes thirty-two decomposition nodes at depth five.	tests/unit/issue_1066_agent_ladder.rs	BTreeMap::from([(0, 1), (1, 2), (2, 4), (3, 8), (4, 16), (5, 32)])
L16	Verify every tested internal node has exactly two children and never three or more.	docs/case-studies/issue-1028/task-decomposition.md	Every internal node has exactly two children
L17	Verify every tested leaf is atomic and independently checkable.	docs/case-studies/issue-1028/task-decomposition.md	every leaf is atomic and independently checkable
L18	Inspect how each decomposition node carries its content-addressed stable identifier.	src/task_decomposition.rs	pub id: String
L19	Verify child paths follow the dotted numeric convention at every recursive depth.	src/task_decomposition.rs	format!("{parent}.{number}")
L20	Verify the node count of a complete depth-five tree is exactly 63 including the root.	tests/unit/issue_1066_agent_ladder.rs	const NODE_COUNT: usize = 63
L21	Inspect the Agent-CLI ladder workflow and verify depth selection supports 0 through 5 and all.	experiments/issue_1028_agent_cli_ladder/run.sh	0|1|2|3|4|5|all) ;;
L22	Verify a single node can be selected by dotted binary path for focused debugging.	experiments/issue_1028_agent_cli_ladder/run.sh	node == filt
L23	Verify the ladder can execute the 32 smallest leaves before moving to larger composite nodes.	experiments/issue_1028_agent_cli_ladder/run.sh	levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
L24	Verify the ladder order for all mode is 32, 16, 8, 4, 2, then the root.	experiments/issue_1028_agent_cli_ladder/run.sh	levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
L25	Verify every selected node runs in a fresh temporary repository copy.	experiments/issue_1028_agent_cli_ladder/run.sh	work=$(mktemp -d)
L26	Verify every selected node uses the real Agent CLI against the real Formal AI server.	experiments/issue_1028_agent_cli_ladder/run.sh	"$AGENT" --model formalai/formal-ai
L27	Verify every selected node requires an observable proof file with its exact node path.	experiments/issue_1028_agent_cli_ladder/run.sh	grep -q "^node_path=$id$" "$proof"
L28	Inspect the committed binary-tree case-study and verify it describes a tree rather than a flat list.	docs/case-studies/issue-1028/task-decomposition.md	This is a complete full binary tree, not a flat list.
L29	Verify the executable ladder formulates exactly 32 distinct atomic leaves.	tests/unit/issue_1066_agent_ladder.rs	const LEAF_COUNT: usize = 32
L30	Verify generated child paths are required to exist in the complete tree.	tests/unit/issue_1066_agent_ladder.rs	assert!(paths.contains(&node.left), "missing {}", node.left)
L31	Inspect the decomposition regression matrix and verify requests are not limited to one fixed wording.	tests/unit/specification/task_decomposition.rs	for (language, prompt) in SPLIT_PROMPTS
L32	Inspect the final evidence-note planner and record the heading used for composed observations.	src/agentic_coding/note_composition.rs	Observed in this session:
EOF

python3 - "$OUT/leaves.tsv" "$NODES" <<'PY'
import sys
from pathlib import Path
leaves = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    leaf, text, criterion_path, criterion_marker = line.split('\t', 3)
    leaves[int(leaf[1:])] = (text, criterion_path, criterion_marker)

def child(path, branch):
    return path + ("." if path else "") + str(branch)

def leaf_index(path):
    bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
    return int(bits, 2) + 1

def emit(path, depth, out):
    if depth == 0:
        text = 'Verify Formal AI supports recursive binary task decomposition from atomic leaves through the complete 32-leaf level.'
        criterion = 'new_composite_effect'
        node_id = 'R'
    elif depth == 5:
        i = leaf_index(path)
        node_id = path
        leaf_text, criterion_path, criterion_marker = leaves[i]
        text = f'Atomic task L{i:02d}: {leaf_text}'
        criterion = 'new_leaf_effect'
    else:
        node_id = path
        bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
        prefix = int(bits, 2)
        span = 2 ** (5 - depth)
        start = prefix * span + 1
        end = (prefix + 1) * span
        text = f'Complete recursive decomposition node {path}, covering atomic tasks L{start:02d}–L{end:02d}; both child nodes must produce independently checkable evidence.'
        criterion = 'new_composite_effect'
    if depth < 5:
        criterion_path = ''
        criterion_marker = ''
    left = child(path, 1) if depth < 5 else ''
    right = child(path, 2) if depth < 5 else ''
    out.append((node_id, depth, text, criterion, left, right, criterion_path, criterion_marker))
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
    node, depth, text, criterion, left, right, criterion_path, criterion_marker = line.split('\t', 7)
    rows.append((node,int(depth),text,criterion,left,right,criterion_path,criterion_marker))
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
  local id depth prompt criterion left right criterion_path criterion_marker row work session_dir server_pid port status proof effect config node_number full_prompt effect_contract verifier_status verifier_verdict
  # Tab is whitespace to Bash, so IFS would collapse the two empty child fields
  # in a leaf row and shift its external criterion into `left`. Translate only
  # for parsing to a non-whitespace separator that preserves empty fields.
  row=${1//$'\t'/$'\x1f'}
  IFS=$'\x1f' read -r id depth prompt criterion left right criterion_path criterion_marker <<< "$row"
  session_dir="$OUT/$id"
  work=$(mktemp -d)
  mkdir -p "$session_dir"
  # A focused replay reuses its stable evidence directory. Remove only the
  # known outputs for this node so a failed attempt cannot retain a prior
  # attempt's proof or effect and appear to have passed.
  rm -f "$session_dir/agent-stream.jsonl"
  rm -f "$session_dir/agent-stderr.log"
  rm -f "$session_dir/formal-ai.log"
  rm -f "$session_dir/proof.md"
  rm -f "$session_dir/effect.lino"
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

  # Cleaning the scratch checkout is not what the ladder measures, and it must
  # never decide a node's verdict. `rm -rf` reports ENOTEMPTY for a directory
  # that gained a file while the walk was inside it, which is what a just-killed
  # server flushing its last write looks like; the trap runs under `set -e`, so
  # that single failure ended the whole run after a node the log had already
  # recorded as PASS. Retry briefly, then leave the directory to the operating
  # system's temporary sweeper rather than failing the node.
  cleanup_one() {
    if [[ -n "${server_pid:-}" ]]; then
      kill -- "-${server_pid}" 2>/dev/null || kill "$server_pid" 2>/dev/null || true
      wait "$server_pid" 2>/dev/null || true
    fi
    local attempt
    for attempt in 1 2 3; do
      rm -rf "$work" 2>/dev/null && return 0
      sleep 1
    done
    echo "cleanup: could not remove $work; leaving it in place" >&2
    return 0
  }
  trap cleanup_one RETURN

  git -C "$ROOT" archive HEAD | tar -x -C "$work"
  git -C "$work" init -q
  git -C "$work" config user.email agent-ladder@example.invalid
  git -C "$work" config user.name agent-ladder
  git -C "$work" add .
  git -C "$work" commit -qm ladder-fixture
  mkdir -p "$work/.agent-ladder"

  if [[ "$depth" -lt 5 ]]; then
    local left_effect_source="${VERIFIED_EFFECTS[$left]:-}"
    local right_effect_source="${VERIFIED_EFFECTS[$right]:-}"
    if [[ ! -s "$left_effect_source" || ! -s "$right_effect_source" ]]; then
      printf '%s\tFAIL\tmissing_current_run_child_effect\n' "$id" >> "$RUN_LOG"
      return 1
    fi
    mkdir -p "$work/.agent-ladder/verified-children"
    cp "$left_effect_source" "$work/.agent-ladder/verified-children/node-$left.lino"
    cp "$right_effect_source" "$work/.agent-ladder/verified-children/node-$right.lino"
    git -C "$work" add .agent-ladder/verified-children
    git -C "$work" commit -qm ladder-verified-child-effects
  fi

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

  if [[ "$depth" -eq 5 ]]; then
    printf -v effect_contract 'Create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=leaf`, and `result=` followed by at least four words that state the task result actually observed in this checkout.' \
      "$id" "$id" "$depth"
  else
    printf -v effect_contract 'Read the committed child effects in `.agent-ladder/verified-children/node-%s.lino` and `.agent-ladder/verified-children/node-%s.lino`. Create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=composite`, `left_child=%s`, `right_child=%s`, `left_result=` followed by the exact left child `result=` value, `right_result=` followed by the exact right child `result=` value, and `result=` followed by at least four words that include both exact child result values and state how they compose.' \
      "$left" "$right" "$id" "$id" "$depth" "$left" "$right"
  fi

  # Built with printf, not interpolated into a double-quoted string: bash does
  # not expand \n there, so the node instructions used to reach the agent as one
  # line with two literal backslash-n in the middle of it.
  printf -v full_prompt '%s\n\nThis is recursive binary-tree node %s at depth %s. Solve only this node'"'"'s task in this fresh temporary repository. Its harness-evaluated completion criterion is: %s. %s Leave supporting evidence in .agent-ladder/node-%s-proof.md. The first line must be exactly node_path=%s and the body must state the concrete result. The harness rejects proof without the separate Git effect. Use web research when it materially improves factual accuracy. Do not claim success without evidence.\n' \
    "$prompt" "$id" "$depth" "$criterion" "$effect_contract" "$id" "$id"

  set +e
  (cd "$work" && \
    FORMAL_AI_API_KEY=local \
    LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --no-summarize-session --compaction-model same \
      --model formalai/formal-ai --permission-mode auto \
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

  effect="$work/agent-ladder-effects/node-${id}.lino"
  set +e
  verifier_verdict=$("$VERIFY_NODE" "$work" "$proof" "$id" "$depth" "$left" "$right" "$criterion_path" "$criterion_marker")
  verifier_status=$?
  set -e
  if [[ "$verifier_status" -ne 0 ]]; then
    printf '%s\tFAIL\t%s\n' "$id" "$verifier_verdict" >> "$RUN_LOG"
    return 1
  fi

  cp "$proof" "$session_dir/proof.md"
  cp "$effect" "$session_dir/effect.lino"
  VERIFIED_EFFECTS["$id"]="$session_dir/effect.lino"
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
PASS requires the external harness to accept a non-hollow proof and a newly
added \`agent-ladder-effects/node-<id>.lino\` Git effect. Composite effects must
consume the immutable, committed effects that both children passed earlier in
this invocation, copy both exact child results, and compose them. Agent exit
zero or a self-authored proof alone never pass.

The \`all\` mode verifies the smallest atomic tasks first (32 leaves), then
16, 8, 4, 2, and finally the root, stopping on the first real failure so the
underlying capability can be repaired before larger composite tasks are tested.
EOF

exit "$failed"
