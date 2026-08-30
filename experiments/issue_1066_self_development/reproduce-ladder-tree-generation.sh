#!/usr/bin/env bash
# Reproduce, then falsify, the three defects that kept the issue #1028 binary-tree
# ladder from ever running a node (issue #1066, acceptance item 2).
#
# The ladder was rewritten from a flat 32-leaf list into a complete binary tree in
# eb16ec1d0. The rewrite never executed: the tree generator crashed on the first
# row it wrote, so `selected.tsv` was never produced and no node was ever
# dispatched. Two further defects sat behind it, both invisible while the crash
# masked them.
#
# Each check below reproduces the broken behaviour first and only then shows the
# committed form behaving differently on the same input, so a regression that
# reintroduces any of the three fails here rather than silently degrading a run.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
LADDER="$ROOT/experiments/issue_1028_agent_cli_ladder/run.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

[[ -f "$LADDER" ]] || { echo "missing $LADDER" >&2; exit 2; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }

# The 32 atomic leaf formulations the ladder writes for itself, lifted straight
# out of the committed heredoc so this experiment cannot drift from the harness.
awk '/^cat > "\$OUT\/leaves.tsv" <<.EOF.$/{flag=1;next} /^EOF$/{flag=0} flag' "$LADDER" > "$WORK/leaves.tsv"
leaf_count=$(wc -l < "$WORK/leaves.tsv" | tr -d ' ')
[[ "$leaf_count" -eq 32 ]] || { echo "expected 32 leaf formulations, got $leaf_count" >&2; exit 1; }

# The tree generator, likewise lifted from the harness rather than restated.
awk '/^python3 - "\$OUT\/leaves.tsv" "\$NODES" <<.PY.$/{flag=1;next} /^PY$/{flag=0} flag' "$LADDER" > "$WORK/generate.py"
[[ -s "$WORK/generate.py" ]] || { echo "could not extract the tree generator" >&2; exit 1; }

fixed_join="'\\t'.join(map(str, r))"
broken_join="'\\t'.join(r)"

grep -Fq "$fixed_join" "$WORK/generate.py" || {
  echo "the tree generator no longer renders every field before joining" >&2
  exit 1
}

# 1. Reproduce: joining the row as-is refuses the integer depth.
python3 - "$WORK/generate.py" "$fixed_join" "$broken_join" "$WORK/broken.py" <<'PY'
import sys
from pathlib import Path
program = Path(sys.argv[1]).read_text()
Path(sys.argv[4]).write_text(program.replace(sys.argv[2], sys.argv[3]))
PY
if python3 "$WORK/broken.py" "$WORK/leaves.tsv" "$WORK/broken.tsv" 2>"$WORK/broken.log"; then
  echo "expected the unfixed join to fail, but it succeeded" >&2
  exit 1
fi
grep -q 'TypeError' "$WORK/broken.log" || { echo "unexpected failure:"; cat "$WORK/broken.log"; exit 1; }
grep -q 'expected str instance, int found' "$WORK/broken.log" || {
  echo "the unfixed join failed for a different reason:" >&2; cat "$WORK/broken.log" >&2; exit 1
}
echo "reproduced: joining the row as-is raises TypeError before a single node is selected"

# ... and the committed form writes the whole tree from the same input.
python3 "$WORK/generate.py" "$WORK/leaves.tsv" "$WORK/tree.tsv"
rows=$(wc -l < "$WORK/tree.tsv" | tr -d ' ')
[[ "$rows" -eq 63 ]] || { echo "expected 63 tree nodes, got $rows" >&2; exit 1; }
python3 - "$WORK/tree.tsv" <<'PY'
import sys
from collections import Counter
from pathlib import Path
rows = [line.split('\t') for line in Path(sys.argv[1]).read_text().splitlines()]
assert all(len(row) == 6 for row in rows), 'every node row has six fields'
depths = Counter(int(row[1]) for row in rows)
assert depths == Counter({0: 1, 1: 2, 2: 4, 3: 8, 4: 16, 5: 32}), depths
paths = [row[0] for row in rows]
assert len(set(paths)) == 63, 'every node path is unique'
assert paths[0] == 'R', paths[0]
for node, depth, _text, _criterion, left, right in rows:
    if int(depth) == 5:
        assert (left, right) == ('', ''), f'leaf {node} claims children'
    else:
        prefix = '' if node == 'R' else node + '.'
        assert (left, right) == (prefix + '1', prefix + '2'), (node, left, right)
        assert left in paths and right in paths, (node, left, right)
leaves = [row for row in rows if int(row[1]) == 5]
assert len({row[2] for row in leaves}) == 32, 'the 32 leaves are distinctly worded'
PY
echo "fixed: the generator writes a complete 63-node binary tree, 32 of them leaves"

# 2. Reproduce: bash `echo` writes a literal backslash-t, so a failure row was
#    one unsplittable field rather than three.
literal=$(echo "id\tFAIL\tmissing_proof")
[[ "$literal" == 'id\tFAIL\tmissing_proof' ]] || { echo "echo unexpectedly expanded \\t" >&2; exit 1; }
[[ "$(printf '%s\n' "$literal" | cut -f2)" == "$literal" ]] || {
  echo "expected the echoed row to have no second field" >&2; exit 1
}
tabbed=$(printf '%s\tFAIL\tmissing_proof' id)
[[ "$(printf '%s\n' "$tabbed" | cut -f2)" == FAIL ]] || { echo "printf did not write a real tab" >&2; exit 1; }
if grep -n 'echo "\$id\\t' "$LADDER"; then
  echo "the ladder still writes run.log rows with echo" >&2
  exit 1
fi
grep -q "printf '%s\\\\tPASS\\\\tdepth=%s\\\\n'" "$LADDER" || {
  echo "the ladder no longer writes its PASS row with printf" >&2; exit 1
}
echo "reproduced and fixed: run.log rows are tab-separated fields, not one literal-backslash blob"

# 3. Reproduce: a double-quoted "\n" is not a newline, so the node instructions
#    reached the agent as a single line with two literal backslash-n in it.
id=1.1.1.1.1
interpolated="task\n\nThis is recursive binary-tree node $id"
[[ "$(printf '%s\n' "$interpolated" | wc -l)" -eq 1 ]] || { echo "expected one line" >&2; exit 1; }
case "$interpolated" in *'\n\n'*) ;; *) echo "expected literal backslash-n" >&2; exit 1 ;; esac
printf -v built '%s\n\nThis is recursive binary-tree node %s' task "$id"
[[ "$(printf '%s\n' "$built" | wc -l)" -eq 3 ]] || { echo "printf did not write real newlines" >&2; exit 1; }
grep -q "printf -v full_prompt" "$LADDER" || { echo "the ladder no longer builds its prompt with printf" >&2; exit 1; }
if grep -n -- '--prompt "\$prompt\\n' "$LADDER"; then
  echo "the ladder still interpolates the node instructions into a double-quoted string" >&2
  exit 1
fi
echo "reproduced and fixed: the node instructions reach the agent as real paragraphs"
