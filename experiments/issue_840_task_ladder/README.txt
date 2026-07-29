Issues #840/#842 task ladder — falsifiable grounded-action quality
==================================================================

WHAT THIS IS
------------
This directory contains the 24-node dataset derived from reports #838, #827,
and #826. Each original request is decomposed through four levels into at least
two smaller tasks. The runner exercises every node against a live
`formal-ai serve`, executes the returned tools as an Agent client would, and
records the complete route/command/answer evidence.

The historical v0.303.0 measurement was 8/24:

  by level  L1 1/3  L2 1/6  L3 2/7  L4 4/8
  by seed   #838 3/10  #827 1/7  #826 4/7

That inversion was the reported defect: Formal AI handled atomic L4 fragments
better than the original L1 requests. The current committed `results.json` is
the strict all-green baseline. L1 and L4 are both 100%, so composition no
longer scores below decomposition.

FILES
-----
  tasks.json          Stable-ID task graph and assertions.
  web_fixtures.json   Deterministic search corpus, including realistic page
                      furniture that must not leak into the answer.
  ladder.py           Real HTTP/tool loop, judge, stable-ID ratchet, and
                      failure-derived learning-proposal writer.
  test_ladder.py      Unit tests for the judge, fixtures, ratchet, and learner.
  run_ladder.sh       Server lifecycle and HTTP/TUI entry point.
  results.json        Last strict measurement; regenerate, never hand-edit.

WHAT A PASS MEANS
-----------------
The judge evaluates only assistant-authored output for answer claims. Raw tool
results remain in the transcript as evidence but cannot satisfy `expect` or
`expect_any`. Every node may also assert:

  expect / expect_any       required answer evidence
  forbid                    answer leakage or false claims
  expect_tool / forbid_tool required and forbidden routes
  command_forbid            unsafe, chained, or early-exit shell fragments

An execution error, generic refusal, or capability-menu fallback always fails.
This closes the false-green case where an empty positive assertion accepted
"не смог определить" and the case where a contextual pronoun silently routed
to web search.

RUNNING IT
----------
  cargo build --release --bin formal-ai
  experiments/issue_840_task_ladder/run_ladder.sh

Useful variants:

  ONLY=827.L4 experiments/issue_840_task_ladder/run_ladder.sh
  MODE=tui ONLY=838.L1 REQUIRE_ALL_PASS=1 \
    experiments/issue_840_task_ladder/run_ladder.sh
  FIXTURES=none ONLY=827 \
    experiments/issue_840_task_ladder/run_ladder.sh
  OUT=/tmp/new.json \
    BASELINE=experiments/issue_840_task_ladder/results.json \
    experiments/issue_840_task_ladder/run_ladder.sh

`FIXTURES=none` is a route-only variant: no search documents are injected, so
synthesis nodes cannot pass on invented evidence. The committed gate uses the
offline corpus for reproducibility. The real Agent CLI journey in
`experiments/agent_cli_e2e/run_issue_840.sh` exercises the same
search/fetch/synthesis procedure through client-owned tools.

EXTENDING THE DATASET
---------------------
Treat every new maintainer report as another durable observation:

1. Decompose the report to the smallest independently falsifiable nodes.
2. Append nodes with new stable IDs; never rename or remove existing IDs.
3. Add answer, route, and command assertions that describe the observed defect.
4. Reproduce the failure, implement the general production rule, and rerun.
5. Advance `results.json` only when every old and new node is green.

The ratchet compares IDs, not aggregate scores. A formerly passing node cannot
be hidden by deleting it or adding an easier node. Appending green nodes is the
normal growth path.

AUTO-LEARNING BOUNDARY
----------------------
Every run writes a sibling `*-learning.json`. Failed nodes become candidates
containing the original stable ID and prompt, assistant output, tools,
commands, error, and exact violated assertions. Passing nodes do not produce
candidates. A candidate remains `awaiting_human_review`; it cannot change
solver behavior or advance the baseline until the complete ladder passes and a
human approves the promotion. This is the task-ladder adapter for Formal AI's
existing failure-derived, review-gated learning pattern.

CI
--
`.github/workflows/task-ladder.yml` builds the release server, tests the judge,
runs all nodes against the committed per-ID baseline, prints the score in the
job summary, and retains both results and learning proposals. The release
workflow additionally runs the reference-agent differential gate, a real
Agent CLI whole-task journey, and a representative OpenCode TUI node.
