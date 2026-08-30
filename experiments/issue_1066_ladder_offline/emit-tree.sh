#!/usr/bin/env bash
# Emit the issue-#1028 ladder as `id<TAB>depth<TAB>prompt<TAB>criterion` rows.
#
# The node texts are the ones `experiments/issue_1028_agent_cli_ladder/run.sh`
# generates; this script exists so the offline harness reads the same tree
# without standing a server up.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
python3 - "$ROOT/experiments/issue_1028_agent_cli_ladder/run.sh" <<'PY'
import re, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
block = re.search(r"<<'EOF'\n(.*?)\nEOF\n", text, re.S).group(1)
leaves = {}
for line in block.splitlines():
    leaf, body = line.split('\t', 1)
    leaves[int(leaf[1:])] = body

def child(path, branch):
    return path + ("." if path else "") + str(branch)

def leaf_index(path):
    bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
    return int(bits, 2) + 1

rows = []

def emit(path, depth):
    if depth == 0:
        node_id, criterion = 'R', 'all_children_pass'
        body = ('Verify Formal AI supports recursive binary task decomposition '
                'from atomic leaves through the complete 32-leaf level.')
    elif depth == 5:
        i = leaf_index(path)
        node_id, criterion = path, 'observable evidence exists'
        body = f'Atomic task L{i:02d}: {leaves[i]}'
    else:
        node_id, criterion = path, 'all_children_pass'
        bits = ''.join('0' if p == '1' else '1' for p in path.split('.'))
        prefix = int(bits, 2)
        span = 2 ** (5 - depth)
        start, end = prefix * span + 1, (prefix + 1) * span
        body = (f'Complete recursive decomposition node {path}, covering atomic '
                f'tasks L{start:02d}–L{end:02d}; both child nodes must produce '
                'independently checkable evidence.')
    rows.append((node_id, depth, body, criterion))
    if depth < 5:
        emit(child(path, 1), depth + 1)
        emit(child(path, 2), depth + 1)

emit('', 0)
for level in range(5, -1, -1):
    for row in rows:
        if row[1] == level:
            print('\t'.join(map(str, row)))
PY
