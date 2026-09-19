# Leaf F-5 — the #840 task ladder, run

Leaf F-5 asks for the #840 task ladder **in five languages including the seven
frontier prompts**. Half of that is possible today and half is not, so both
halves are recorded.

## What was run

```sh
PORT=8921 OUT=…/self-use/ladder-840/results-wave-f.json \
  experiments/issue_840_task_ladder/run_ladder.sh
```

`formal-ai 0.350.0` built from `74875c1b`, offline `web_fixtures.json`, port 8921,
a fresh server, results written into this directory so
`experiments/issue_840_task_ladder/results.json` is untouched.

## Numbers

```
TOTAL 24/24 passed
  L1: 3/3   L2: 6/6   L3: 7/7   L4: 8/8
  #826: 7/7   #827: 7/7   #838: 10/10
```

0 review-gated learning candidates. This reproduces the committed baseline
exactly; nothing regressed and nothing moved.

## What was not run, and why

The ladder has **24 nodes in two languages** — 10 English, 14 Russian. There are
no Hindi, Chinese or Spanish nodes, and none of the seven frontier prompts of
#1087 is a node. Adding them is plan 10 leaf 10-21's deliverable, which wave F
leaf F-5 depends on (`I9 (10-21)`) and which has not landed; appending them here
would also mean editing `experiments/issue_840_task_ladder/tasks.json`, outside
this session's file namespace. So **the five-language and frontier half of F-5 is
recorded as not run**, not as delivered.

## The number worth keeping

| measurement | prompts | passed |
| --- | --- | --- |
| #840 ladder, committed nodes, committed offline fixtures | 24 | **24 (100 %)** |
| wave F held-out prompts, live | 113 | **6 (5 %)** |

The same binary, on the same afternoon. The ladder is all green because its 24
nodes and their fixtures are the ones the system has been tuned against; every
prompt in the wave F corpora was written to be absent from `data/seed/` and
`src/`. The gap between the two rows is the measurement plan 00 §1 is about, and
it is the reason leaf F-5 asks for five languages and the frontier prompts rather
than for another run of the nodes that already pass.

A ladder that is 100 % is not evidence of capability; it is evidence that the
ladder stopped being a question. Appending the held-out nodes — ids appended,
never renamed, per the ladder README — is what would make it one again.
