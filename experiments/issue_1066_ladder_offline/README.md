# Issue #1066 — the agent ladder, run offline and judged

`experiments/issue_1028_agent_cli_ladder/run.sh` is the ground truth for issue
#1066's second acceptance item: the sixty-three-node binary tree has to complete
at depth five through the real Agent CLI. A release build plus a real CLI turn
per node costs about a minute a node, which is the wrong loop to debug a
capability gap in.

This directory runs the same sixty-three node prompts through the same planner
offline, and — more importantly — judges what the nodes actually produced.

## Why the mechanical criterion was not enough

The ladder calls a node passed when the Agent CLI exits 0 and
`.agent-ladder/node-<id>-proof.md` exists, is non-empty, and its first line is
`node_path=<id>`. Every one of those is checkable without reading the proof, and
that is the point: the harness is not supposed to grade prose.

Run against this repository, that criterion reported **63/63 PASS** — over
thirty-two proof files that said nothing. A file reading

```text
node_path=1.1.1.1.1

Sub-tasks:
```

satisfies the marker, is non-empty, and exits 0. It is also not evidence of
anything. The ladder was green because the criterion could not tell a proof from
a heading.

## What the scripts do

- `emit-tree.sh` re-derives the sixty-three `id / depth / prompt / criterion`
  rows from `experiments/issue_1028_agent_cli_ladder/run.sh` itself, so the
  offline run cannot drift from the tree it claims to be running.
- `run.sh` drives each prompt through
  `examples/issue_1066_ladder_node_offline`, which advertises the same fourteen
  tools the Agent CLI does and executes each planned call against a throwaway
  `git archive HEAD` copy of the tree.
- `judge-proof.py` reads what is *under* the marker line and fails a node whose
  proof is a heading with no list, a word naming the work product ("the
  result"), a report that the write step failed, or fewer than four words.
- `falsify-node-capabilities.sh` neutralises each fix issue #1066 added — one
  early return, in the one function that decides — and asserts the matching test
  goes red, then restores the file and asserts the whole set goes green again.

```bash
cargo build --example issue_1066_ladder_node_offline
bash experiments/issue_1066_ladder_offline/run.sh          # all 63 nodes
bash experiments/issue_1066_ladder_offline/run.sh 1.1.1.1.1 # one node
experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh
```

Every proof is kept under `$OUT/proofs/` (default
`/tmp/issue-1066-ladder-offline`) so a reader can check the judgement rather
than trust it.
