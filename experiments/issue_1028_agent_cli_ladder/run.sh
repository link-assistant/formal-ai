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
# Issue #1085 (D4): every leaf's edit is compiled by verify-node.sh. One target
# directory for the whole run keeps that incremental after the first leaf.
export LADDER_CARGO_TARGET_DIR="${LADDER_CARGO_TARGET_DIR:-$STAGE/target}"

NODES="$OUT/tree.tsv"
RUN_LOG="$OUT/run.log"
: > "$NODES"
: > "$RUN_LOG"
declare -A VERIFIED_EFFECTS=()

# Issue #1085 (D2.3, D4): the leaf table is committed beside this script so
# the unit tests, the 32 rule files under rules/ and this run read one source.
cp "$ROOT/experiments/issue_1028_agent_cli_ladder/leaves.tsv" "$OUT/leaves.tsv"

python3 - "$OUT/leaves.tsv" "$NODES" <<'PY'
import sys
from pathlib import Path
leaves = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    leaf, text, change_path, change_marker, change_guard, requirement = line.split('\t', 5)
    leaves[int(leaf[1:])] = (text, change_path, change_marker, change_guard, requirement)

def child(path, branch):
    return path + ("." if path else "") + str(branch)

def leaf_index(path):
    bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
    return int(bits, 2) + 1

def requirement_text(start, end):
    listed = ' '.join(f'({k}) {leaves[k][4]}' for k in range(start, end + 1))
    return ('Deliver these requirements in this repository; each names behaviour '
            'or a declaration, never a file, so find the declaration in the '
            'source tree first: ' + listed)
def emit(path, depth, out):
    if depth == 0:
        # Issue #1085 (D4): the root and every node down to depth 3 are
        # requirement-shaped; the prompt names behaviour, never a file.
        text = 'Root task: ' + requirement_text(1, 32)
        criterion = 'requirement_changes'
        node_id = 'R'
    elif depth == 5:
        i = leaf_index(path)
        node_id = path
        leaf_text, criterion_path, criterion_marker, criterion_guard, _requirement = leaves[i]
        text = f'Atomic task L{i:02d}: {leaf_text}'
        criterion = 'tracked_source_change'
    else:
        node_id = path
        bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
        prefix = int(bits, 2)
        span = 2 ** (5 - depth)
        start = prefix * span + 1
        end = (prefix + 1) * span
        if depth <= 3:
            text = f'Decomposition node {path}: ' + requirement_text(start, end)
            criterion = 'requirement_changes'
        else:
            text = f'Complete recursive decomposition node {path}, covering atomic tasks L{start:02d}–L{end:02d}; both child nodes must produce independently checkable evidence.'
            criterion = 'new_composite_effect'
    if depth < 5:
        criterion_path = ''
        criterion_marker = ''
        criterion_guard = ''
    left = child(path, 1) if depth < 5 else ''
    right = child(path, 2) if depth < 5 else ''
    out.append((node_id, depth, text, criterion, left, right, criterion_path,
                criterion_marker, criterion_guard))
    if depth < 5:
        emit(child(path,1), depth+1, out)
        emit(child(path,2), depth+1, out)

rows=[]
emit('',0,rows)
# `depth` is an int, and str.join refuses a non-str item, so joining the row
# straight raised TypeError before a single node was ever selected. Render
# every field before joining rather than trusting the tuple to be all strings.
# Empty optional fields belong to the in-memory row, but emitting their trailing
# separators makes the committed TSV fail git's whitespace check. Readers pad
# omitted tail fields back to the eight-field schema below.
Path(sys.argv[2]).write_text(
    '\n'.join('\t'.join(map(str, r)).rstrip('\t') for r in rows) + '\n'
)
PY

python3 - "$NODES" "$TREE_DEPTH" "$NODE_FILTER" > "$OUT/selected.tsv" <<'PY'
import sys
from pathlib import Path
rows=[]
for line in Path(sys.argv[1]).read_text().splitlines():
    fields = line.split('\t', 8)
    if not 6 <= len(fields) <= 9:
        raise ValueError(f'node row has {len(fields)} fields, expected 6 through 9: {line!r}')
    fields.extend([''] * (9 - len(fields)))
    (node, depth, text, criterion, left, right,
     criterion_path, criterion_marker, criterion_guard) = fields
    rows.append((node, int(depth), text, criterion, left, right,
                 criterion_path, criterion_marker, criterion_guard))
mode=sys.argv[2]
filt=sys.argv[3]
levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
for level in levels:
    for row in rows:
        node, depth, *_ = row
        in_focused_subtree = (
            not filt
            or filt == 'R'
            or node == filt
            or node.startswith(filt + '.')
        )
        if depth == level and in_focused_subtree:
            print('\t'.join(map(str,row)).rstrip('\t'))
PY

selected_count=$(wc -l < "$OUT/selected.tsv" | tr -d ' ')
expected=1
if [[ "$TREE_DEPTH" = all ]]; then
  if [[ -z "$NODE_FILTER" || "$NODE_FILTER" = R ]]; then
    expected=63
  else
    filter_depth=$(awk -F. '{ print NF }' <<< "$NODE_FILTER")
    expected=$(( (1 << (6 - filter_depth)) - 1 ))
  fi
elif [[ -n "$NODE_FILTER" ]]; then
  expected=1
else
  expected=$((1 << TREE_DEPTH))
fi
[[ "$selected_count" -eq "$expected" ]] || { echo "expected $expected selected nodes, got $selected_count" >&2; exit 1; }

run_one() {
  local id depth prompt criterion left right criterion_path criterion_marker criterion_guard row work session_dir server_pid port status proof effect config node_number full_prompt effect_contract verifier_status verifier_verdict
  # Tab is whitespace to Bash, so IFS would collapse the two empty child fields
  # in a leaf row and shift its external criterion into `left`. Translate only
  # for parsing to a non-whitespace separator that preserves empty fields.
  row=${1//$'\t'/$'\x1f'}
  IFS=$'\x1f' read -r id depth prompt criterion left right criterion_path criterion_marker criterion_guard <<< "$row"
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

  local child_diffs="" leaf_span=""
  if [[ "$depth" -eq 4 ]]; then
    local left_effect_source="${VERIFIED_EFFECTS[$left]:-}"
    local right_effect_source="${VERIFIED_EFFECTS[$right]:-}"
    if [[ ! -s "$left_effect_source" || ! -s "$right_effect_source" ]]; then
      printf '%s\tFAIL\tmissing_current_run_child_effect\n' "$id" >> "$RUN_LOG"
      return 1
    fi
    # Issue #1085 (D4): a composite must also merge both children's diffs.
    if [[ ! -s "$OUT/$left/change.diff" || ! -s "$OUT/$right/change.diff" ]]; then
      printf '%s\tFAIL\tmissing_current_run_child_diff\n' "$id" >> "$RUN_LOG"
      return 1
    fi
    child_diffs="$OUT/$left/change.diff $OUT/$right/change.diff"
    mkdir -p "$work/.agent-ladder/verified-children"
    cp "$left_effect_source" "$work/.agent-ladder/verified-children/node-$left.lino"
    cp "$right_effect_source" "$work/.agent-ladder/verified-children/node-$right.lino"
    git -C "$work" add .agent-ladder/verified-children
    git -C "$work" commit -qm ladder-verified-child-effects
  elif [[ "$depth" -le 3 ]]; then
    leaf_span=$(python3 - "$id" "$depth" <<'PY'
import sys
node, depth = sys.argv[1], int(sys.argv[2])
if node == 'R':
    print('1-32')
else:
    bits = ''.join('0' if p == '1' else '1' for p in node.split('.'))
    span = 2 ** (5 - depth)
    start = int(bits, 2) * span + 1
    print(f'{start}-{start + span - 1}')
PY
)
  fi

  setsid env FORMAL_AI_AGENT_MODE=1 FORMAL_AI_TRACE_REQUESTS=1 \
    FORMAL_AI_MEMORY_PATH="$work/.git/formal-ai-memory/memory.lino" \
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
    printf -v effect_contract 'Apply the change to the tracked file `%s` itself -- the file has to end up modified in the Git worktree, and nothing else may change. Then create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=leaf`, and `result=` followed by at least four words that state the change you made and that contain the exact text %s.' \
      "$criterion_path" "$id" "$id" "$depth" "$criterion_marker"
  elif [[ "$depth" -le 3 ]]; then
    printf -v effect_contract 'Apply every listed requirement to the tracked source files themselves -- locate each declaration in the repository first (for example with grep), change only the files those requirements touch, and keep them valid Rust. Then create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=requirement`, and `result=<the files you changed and what changed in each, at least four words>`.' \
      "$id" "$id" "$depth"
  else
    printf -v effect_contract 'Read the committed child effects in `.agent-ladder/verified-children/node-%s.lino` and `.agent-ladder/verified-children/node-%s.lino`. Inspect both files before writing anything. Extract each raw child value with `sed -n "s/^result=//p" FILE` or an equivalent command that returns undecorated file bytes. Treat only the single line beginning exactly `result=` as that child result. Do not copy tool-rendered line numbers, `<file>` wrappers, or any other fields. Create `agent-ladder-effects/node-%s.lino` with these exact field lines: `node_path=%s`, `node_depth=%s`, `node_kind=composite`, `left_child=%s`, `right_child=%s`, `left_result=` followed by the exact left child `result=` value, `right_result=` followed by the exact right child `result=` value, and `result=` followed by at least four words that include both exact child result values and state how they compose.' \
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
    "$AGENT" --no-summarize-session --compaction-models "(same)" \
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

  # Agent CLI terminates stream-json output with a presentation-only blank
  # line. Keep the committed JSONL canonical so every line is a JSON record
  # and the generated evidence passes Git's whitespace check.
  sed -i '${/^$/d;}' "$session_dir/agent-stream.jsonl"

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
  verifier_verdict=$(LADDER_LEAVES="$OUT/leaves.tsv" LADDER_LEAF_SPAN="$leaf_span" LADDER_CHILD_DIFFS="$child_diffs" \
    "$VERIFY_NODE" "$work" "$proof" "$id" "$depth" "$left" "$right" "$criterion_path" "$criterion_marker" "$criterion_guard")
  verifier_status=$?
  set -e
  if [[ "$verifier_status" -ne 0 ]]; then
    printf '%s\tFAIL\t%s\n' "$id" "$verifier_verdict" >> "$RUN_LOG"
    return 1
  fi

  cp "$proof" "$session_dir/proof.md"
  cp "$effect" "$session_dir/effect.lino"
  [[ -f "$work/.agent-ladder/verify.tsv" ]] && cp "$work/.agent-ladder/verify.tsv" "$session_dir/verify.tsv"
  if [[ "$depth" -eq 5 ]]; then
    git -C "$work" diff -- "$criterion_path" > "$session_dir/change.diff"
  elif [[ "$depth" -eq 4 ]]; then
    cat "$OUT/$left/change.diff" "$OUT/$right/change.diff" > "$session_dir/change.diff"
  else
    git -C "$work" diff > "$session_dir/change.diff"
  fi
  VERIFIED_EFFECTS["$id"]="$session_dir/effect.lino"
  printf '%s\tPASS\tdepth=%s\n' "$id" "$depth" >> "$RUN_LOG"
}

failed=0
current_level=""
level_failed=0
# Issue #1085 (D4): a level is finished before the run stops, so the record
# says which level passed completely; in `all` mode a failed level ends the run
# because the level above builds on it.
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  node=$(printf '%s\n' "$line" | cut -f1)
  level=$(printf '%s\n' "$line" | cut -f2)
  if [[ "$current_level" != "$level" ]]; then
    if [[ -n "$current_level" && "$level_failed" -eq 1 && "$TREE_DEPTH" == all ]]; then
      echo "level $current_level failed; not attempting level $level" | tee -a "$RUN_LOG"
      break
    fi
    current_level="$level"
    level_failed=0
  fi
  echo "=== $node ===" | tee -a "$RUN_LOG"
  if run_one "$line"; then :; else failed=$((failed + 1)); level_failed=1; fi
done < "$OUT/selected.tsv"
deepest=none
leaf_level_passed=0
leaf_nodes_selected=$(awk -F'\t' '$2 == 5' "$OUT/selected.tsv" | wc -l | tr -d ' ')
leaf_nodes_passing=$(grep -c $'\tPASS\tdepth=5$' "$RUN_LOG" || true)
for level in 5 4 3 2 1 0; do
  selected_at_level=$(awk -F'\t' -v level="$level" '$2 == level' "$OUT/selected.tsv" | wc -l | tr -d ' ')
  [[ "$selected_at_level" -gt 0 ]] || continue
  passed_at_level=$(grep -c $'\tPASS\tdepth='"$level"'$' "$RUN_LOG" || true)
  if [[ "$passed_at_level" -eq "$selected_at_level" ]]; then
    deepest="$level"
    [[ "$level" -eq 5 ]] && leaf_level_passed=1
  else
    break
  fi
done
cat > "$OUT/ladder-result.lino" <<EOF
ladder_result
  requested_depth "$TREE_DEPTH"
  node_filter "${NODE_FILTER:-none}"
  selected_nodes "$selected_count"
  failures "$failed"
  deepest_passing_level "$deepest"
  leaf_nodes_selected "$leaf_nodes_selected"
  leaf_nodes_passing "$leaf_nodes_passing"
EOF
python3 - "$OUT" "$TREE_DEPTH" "${NODE_FILTER:-none}" "$selected_count" "$failed" "$deepest" \
  "$leaf_nodes_passing" "$leaf_nodes_selected" <<'PY'
import sys
from pathlib import Path
out = Path(sys.argv[1])
depth, node_filter, selected, failed, deepest, leaf_passing, leaf_selected = sys.argv[2:9]
rows = []
for line in (out / 'run.log').read_text().splitlines():
    parts = line.split('\t')
    if len(parts) != 3:
        continue
    node, verdict, detail = parts
    report = {}
    verify = out / node / 'verify.tsv'
    if verify.exists():
        for entry in verify.read_text().splitlines():
            key, _, value = entry.partition('\t')
            report[key] = value
    tests = ', '.join(f'{k[6:]} {v}' for k, v in report.items() if k.startswith('tests:')) or '-'
    rows.append(f"| {node} | {verdict} | {detail} | {report.get('compile', '-')} | {tests} | {report.get('diff_lines', '-')} |")
table = '\n'.join(rows) if rows else '| - | - | no node ran | - | - | - |'
(out / 'README.md').write_text(f"""# Agent CLI binary-tree ladder run

- requested depth: {depth}
- node filter: {node_filter}
- selected nodes: {selected}
- failures: {failed}
- deepest level whose nodes all passed: {deepest}
- leaf nodes passing: {leaf_passing} of {leaf_selected}

| node | verdict | detail | compile | unit tests | diff lines |
| --- | --- | --- | --- | --- | --- |
{table}

The canonical decomposition is a complete binary tree: depth 0 has 1 node,
depth 1 has 2, depth 2 has 4, depth 3 has 8, depth 4 has 16, and depth 5 has 32.
Each selected node runs in a fresh temporary repository copy against the real
`@link-assistant/agent` CLI and a local `formal-ai serve --agent-mode`.

Every leaf is *change-shaped*: a member insertion, a literal replacement or an
identifier rename in a tracked source (the committed `leaves.tsv`, each also a
link-edit rule under `rules/`). PASS requires the worktree to show exactly that
one file modified, the marker absent from `HEAD`, the anchor still present, the
file formatting, `cargo check --lib` compiling it, and `cargo test --test unit`
passing for its module (issue #1085 D4). A depth-4 composite must compose both
verified child effects and both children's diffs must apply to one tree and
compile. Depth 3 and above are *requirement-shaped*: the prompt lists behaviour
and declarations, never files; every leaf marker under the node must be
present, only those files may change, and the tree must compile and pass the
tests of every touched module. An effect file that merely describes a change
never passes.

The `all` mode verifies the 32 leaves first, then 16, 8, 4, 2, and the root; a
level is finished before the run stops, and a failed level ends the run. The
deepest level whose nodes all passed is written to `ladder-result.lino` and
compared with `data/meta/ladder-ratchet.lino` by the workflow.
""")
PY
# The run measures; the ratchet step decides. Exiting non-zero on any failed
# node made the job red for every leaf Formal AI cannot yet change, which says
# nothing about whether it got worse (issue #1085 D4).
if [[ "$TREE_DEPTH" == all && "$leaf_level_passed" -eq 1 ]]; then
  echo "every leaf passed" | tee -a "$RUN_LOG"
fi
exit 0
