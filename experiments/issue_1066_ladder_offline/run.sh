#!/usr/bin/env bash
# Drive every node of the issue-#1028 ladder through the planner offline.
#
# `experiments/issue_1028_agent_cli_ladder/run.sh` is the ground truth, but a
# release build plus a real Agent CLI per node costs about a minute a node. This
# harness runs the same node prompts through `examples/issue_1066_ladder_node_offline`,
# which advertises the same fourteen tools and executes each planned call against
# a throwaway copy of the tree, and reports PASS/FAIL per node using the same
# criterion `run.sh` uses -- a non-empty `.agent-ladder/node-<id>-proof.md`
# whose first line is `node_path=<id>` -- *and* the judgement that criterion
# cannot make.
#
# The mechanical criterion is what reported sixty-three green nodes over
# thirty-two proof files that said nothing. `judge-proof.py` reads what is under
# the marker line and fails a node whose proof is a heading with no list, a word
# naming the work product, or a report that the step failed. Every proof is kept
# under `$OUT/proofs/` so a reader can check the judgement rather than trust it.
#
# Usage:
#   bash experiments/issue_1066_ladder_offline/run.sh [node-id ...]
#
# With no arguments every node of the complete depth-five tree is run.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT="${OUT:-/tmp/issue-1066-ladder-offline}"
TURNS="${TURNS:-12}"
EXAMPLE="$ROOT/target/debug/examples/issue_1066_ladder_node_offline"

[[ -x "$EXAMPLE" ]] || { echo "build first: cargo build --example issue_1066_ladder_node_offline" >&2; exit 2; }

mkdir -p "$OUT" "$OUT/proofs"
NODES="$OUT/tree.tsv"
bash "$ROOT/experiments/issue_1066_ladder_offline/emit-tree.sh" > "$NODES"

wanted=("$@")
pass=0; fail=0
: > "$OUT/summary.tsv"
while IFS=$'\t' read -r id depth prompt criterion; do
  if [[ ${#wanted[@]} -gt 0 ]]; then
    keep=0
    for want in "${wanted[@]}"; do [[ "$want" == "$id" ]] && keep=1; done
    [[ $keep -eq 1 ]] || continue
  fi
  work="$OUT/work/$id"
  rm -rf "$work"; mkdir -p "$work/.agent-ladder"
  git -C "$ROOT" archive HEAD | tar -x -C "$work"
  "$EXAMPLE" --task "$prompt" --node "$id" --depth "$depth" \
    --criterion "$criterion" --workspace "$work" --turns "$TURNS" \
    > "$OUT/$id.log" 2>&1
  proof="$work/.agent-ladder/node-$id-proof.md"
  [[ -f "$proof" ]] && cp "$proof" "$OUT/proofs/node-$id-proof.md"
  verdict=$(python3 "$ROOT/experiments/issue_1066_ladder_offline/judge-proof.py" "$proof" "$id")
  if [[ "$verdict" == ok ]]; then
    printf '%s\tPASS\n' "$id" >> "$OUT/summary.tsv"; pass=$((pass+1))
  else
    printf '%s\tFAIL\t%s\n' "$id" "$verdict" >> "$OUT/summary.tsv"; fail=$((fail+1))
  fi
  rm -rf "$work"
done < "$NODES"

echo "pass=$pass fail=$fail"
grep -c FAIL "$OUT/summary.tsv" >/dev/null && grep FAIL "$OUT/summary.tsv" | head -70
[[ "$fail" -eq 0 ]]
