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
L01	Extract the four concrete requirements of issue #1028 and record them with evidence.
L02	Inspect scripts/apt-install-with-retry.sh and record its retry inputs and defaults.
L03	Inspect tests/unit/ci-cd/issue_1021.rs and identify the reusable retry harness.
L04	Inspect the Agentic CLI workflow Xvfb step and record its retry budget variables.
L05	Calculate the old 3-attempt, 90-second, 5-second-delay worst-case schedule.
L06	Calculate the 1:2:4 geometric allocation for the remaining retry budget.
L07	Verify callers without TEST_BUDGET_SECONDS retain fixed per-attempt deadline behavior.
L08	State and verify the invariant that retry deadlines plus delays fit the step budget.
L09	Add or verify the focused geometric deadline allocation test.
L10	Add or verify a deterministic slow-mirror stand-in without network access.
L11	Prove the old flat retry schedule fails the slow-mirror fixture.
L12	Prove the budget-aware escalating schedule succeeds on that fixture.
L13	Verify retry delays are reserved before attempt deadlines are allocated.
L14	Verify later retries receive strictly more execution time than earlier retries.
L15	Verify the first retry share is positive and smaller than the final share.
L16	Verify failure diagnostics report the actual deadline used by the attempt.
L17	Verify persistent non-timeout apt failure preserves apt's exit status.
L18	Verify timeout status 124 is distinguished from ordinary apt failures.
L19	Review retry-wrapper comments for a general geometric algorithm.
L20	Review the Xvfb workflow and wrapper budget interface for consistency.
L21	Complete the issue-1028 case-study evidence with concrete artifact paths.
L22	Complete the changelog fragment for the retry scheduling fix.
L23	Run bash syntax validation on scripts/apt-install-with-retry.sh.
L24	Run the focused issue-1028 Rust tests and record the result.
L25	Inspect the working diff and remove unrelated issue changes.
L26	Produce a PR summary explaining the reusable retry generalization.
L27	Add requirement-traceability evidence for all four issue requirements.
L28	Validate the decomposition artifact has 32 unique leaf nodes.
L29	Round-trip the task-decomposition artifact through the Links Notation path.
L30	Classify any ladder failure from observable evidence.
L31	Generalize any discovered capability gap and add a differently worded regression.
L32	Produce the final self-coding evidence bundle with outcomes and session IDs.
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
        text = 'Solve issue #1028 end-to-end: improve apt retry scheduling and prove the coding capability recursively.'
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
        text = f'Complete recursive subtask {path}, covering leaf requirements L{start:02d}–L{end:02d}; both child subtasks must produce checkable evidence.'
        criterion = 'all_children_pass'
    out.append((node_id, depth, text, criterion, child(path,1), child(path,2) if depth < 5 else ''))
    if depth < 5:
        emit(child(path,1), depth+1, out)
        emit(child(path,2), depth+1, out)

rows=[]
emit('',0,rows)
Path(sys.argv[2]).write_text('\n'.join('\t'.join(r) for r in rows)+'\n')
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
            if not filt or any(level == int(mode) for _ in [0]):
                print('\t'.join(map(str,row)))
PY

selected_count=$(wc -l < "$OUT/selected.tsv" | tr -d ' ')
expected=1
if [[ "$TREE_DEPTH" = all ]]; then
  expected=63
elif [[ -z "$NODE_FILTER" ]]; then
  expected=$((1 << TREE_DEPTH))
fi
[[ "$selected_count" -eq "$expected" ]] || { echo "expected $expected selected nodes, got $selected_count" >&2; exit 1; }

run_one() {
  local id depth prompt criterion work session_dir server_pid port status proof config
  IFS=$'\t' read -r id depth prompt criterion _left _right <<< "$1"
  session_dir="$OUT/$id"
  work=$(mktemp -d)
  mkdir -p "$session_dir"

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

  port=$((BASE_PORT + 10#$(python3 - <<PY
print(abs(hash('$id')) % 1000)
PY
)))

  setsid env FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.agent-ladder/memory.lino" \
    FORMAL_AI_DREAMING=0 "$BIN" serve --agent-mode --host 127.0.0.1 --port "$port" \
    >"$session_dir/formal-ai.log" 2>&1 &
  server_pid=$!

  if ! curl -fsS --retry 30 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$port/health" >/dev/null; then
    echo "$id\tFAIL\tformal_ai_server_start" >> "$RUN_LOG"
    tail -100 "$session_dir/formal-ai.log" >&2 || true
    return 1
  fi

  config="$(printf '{\"provider\":{\"formalai\":{\"name\":\"Formal AI\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:%s/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Formal AI\"}}}},\"model\":\"formalai/formal-ai\"}' "$port")"

  set +e
  (cd "$work" && \
    FORMAL_AI_API_KEY=local \
    LINK_ASSISTANT_AGENT_CONFIG_CONTENT="$config" \
    "$AGENT" --model formalai/formal-ai --permission-mode auto \
      --output-format stream-json --compact-json --disable-stdin \
      --prompt "$prompt\n\nThis is recursive binary-tree node $id at depth $depth. Solve only this node's task in this fresh temporary repository. Its completion criterion is: $criterion. Leave observable evidence in .agent-ladder/node-${id}-proof.md. The first line must be exactly node_path=$id. Use web research when it materially improves factual accuracy. Do not claim success without evidence.") \
      >"$session_dir/agent-stream.jsonl" 2>"$session_dir/agent-stderr.log"
  status=$?
  set -e
  if [[ "$status" -ne 0 ]]; then
    echo "$id\tFAIL\tagent_exit_$status" >> "$RUN_LOG"
    return 1
  fi

  proof="$work/.agent-ladder/node-${id}-proof.md"
  if [[ ! -s "$proof" ]]; then
    echo "$id\tFAIL\tmissing_proof" >> "$RUN_LOG"
    return 1
  fi
  if ! grep -q "^node_path=$id$" "$proof"; then
    echo "$id\tFAIL\tbad_proof_marker" >> "$RUN_LOG"
    return 1
  fi

  cp "$proof" "$session_dir/proof.md"
  echo "$id\tPASS\tdepth=$depth" >> "$RUN_LOG"
}

failed=0
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  node=$(printf '%s\n' "$line" | cut -f1)
  echo "=== $node ===" | tee -a "$RUN_LOG"
  if run_one "$line"; then :; else failed=1; break; fi
done < "$OUT/selected.tsv"

cat > "$OUT/README.md" <<EOF
# Issue #1028 recursive Agent-CLI tree run

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
