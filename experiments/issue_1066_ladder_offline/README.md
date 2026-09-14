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

## Observed

Each number below is `judge-proof.py`'s verdict over all sixty-three proofs,
re-judged with the *current* judge so the comparison is like for like — an
earlier judge scoring an earlier run is not a measurement of the code.

| State of `src/` | `ok` | hollow |
| --- | --- | --- |
| Before the request-block fixes (gaps 22, 23) | 54 | 9 |
| After them | 57 | 6 |
| After the query-scope fixes (gaps 24, 25) | 62 | 1 |
| After the quoted-match fix (gaps 26, 27) | 63 | 0 |

The last row is the whole harness green, and the proof it turns is worth naming:
node `2.2.1.1.2` answers with fifty lines `grep` matched under `scripts/`, and
before gaps 26 and 27 it filed them under "The command failed" because
`install.sh` prints the words "was not found" when the `code` CLI is missing.

Judging that node also found a hole in the judge. `judge-proof.py` had no marker
for the renderer's own failure sentence, and caught the bad proof only by
accident -- one of the quoted lines is `except Exception as error:`, which its
`error:` marker matched. So the marker list now carries `the command failed` in
each language the renderer says it in, and the markers are read against what a
proof says *before* it starts citing places, exactly as
`src/agentic_coding/tool_result.rs` reads a tool result. Re-judging every run
above with the corrected judge reproduces all three earlier rows unchanged, and
all eight hollow verdicts in the committed 32-leaf run stay hollow: the
correction rescues quoted evidence and nothing else.

The real 32-leaf ladder run committed under
`docs/case-studies/issue-1066/ladder-before-fix/` is the same measurement over
the Agent CLI rather than the offline harness, and it starts in the same place:
24 `ok` and 8 hollow. Every hollow verdict there is a node that answered with
the sentence a lookup returns when it comes back with no content; the one in the
offline harness's last row is the other kind, a search that ran and was filed as
a failure. `docs/case-studies/issue-1028/agent-tree-run/` holds the re-run on the
fixed code, judged the same way.

A passing judge is not the same as a good proof, and this directory does not
claim it is: fourteen of the thirty-two leaves in that run record a correct
verdict about their own task label instead of doing the task, and `judge-proof.py`
accepts all fourteen because it judges shape and never sees the task. That gap is
written up in `docs/case-studies/issue-1066/README.md` under "What is still
hollow, and why the judge does not see it".
