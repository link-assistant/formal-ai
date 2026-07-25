Issue #840 task ladder — measured baseline for Formal AI answer quality
======================================================================

WHAT THIS IS
------------
A 24-node task dataset derived from the three seed reports (#838, #827, #826),
decomposed four levels deep (each task split into >= 2 subtasks), plus a runner
that drives every node against a live `formal-ai serve` and records pass/fail.

The point is to replace "the maintainer reports a phrasing, we fix that
phrasing" with a dataset that can be re-measured after every change.

FILES
-----
  tasks.json      The ladder. 24 nodes: L1 = original seed prompt, L4 = smallest
                  atomic step. Each node carries `expect` (all substrings must
                  appear) and `forbid` (none may appear), plus a `note` saying
                  which standard from #840 it exercises.
  run_ladder.sh   Boots the server with agent mode, runs a real agentic loop
                  (executes returned bash commands in a sandbox and feeds the
                  output back, as opencode does), writes results.json.
  results.json    Output of the last run. Regenerate; do not hand-edit.

USAGE
-----
  cargo build --release
  experiments/issue_840_task_ladder/run_ladder.sh
  ONLY=838 experiments/issue_840_task_ladder/run_ladder.sh   # filter by id
  MODE=tui ONLY=838.L1 experiments/issue_840_task_ladder/run_ladder.sh
      # same node through the real OpenCode PTY; writes transcript, frames,
      # asciicast, static SVG, and animated SVG beside the results

Knobs: BIN, PORT, TASKS, OUT, ONLY, SANDBOX, SANDBOX_KEEP, MODE,
TUI_ARTIFACT_DIR, REQUIRE_ALL_PASS.
The measurement harness exits 0 by default. Set REQUIRE_ALL_PASS=1 to make a
selected TUI subset a CI gate.

BASELINE @ v0.303.0 (main 1873e873), 2026-07-25
-----------------------------------------------
  TOTAL  8/24
  by level:  L1 1/3   L2 1/6   L3 2/7   L4 4/8
  by seed:   #838 3/10   #827 1/7   #826 4/7

Note the inversion: the system does BETTER on the smallest subtasks (L4 4/8)
than on the original user-facing requests (L1 1/3). Decomposition is not the
weak point; composing a whole answer is.

THE HEADLINE FINDING
--------------------
Three phrasings of one intent, three different routes:

  838.L1    "Find hive-mind-control center folder on my desktop"  -> bash    PASS
  838.L3.a  "Search hive-mind-control-center on my desktop"        -> websearch  FAIL
  838.L3.b  "Find hive-mind-control center folder on desktop"      -> websearch  FAIL

The only differences are the verb (Find/Search) and the possessive (my). This
is the routing asymmetry #840 describes, reproduced deterministically.

Caveat on 838.L1's PASS: it passes only because `find` happened to echo a
matching path. The command it ran still uses five guessed globs and
`-print -quit`, and the final message pastes the raw command back at the user.
It satisfies the substring assertion, not the standard. Assertions here are a
floor, not the definition of done.

A FIRST RUN WITHOUT AGENT MODE SCORED 4/24 AND WAS WRONG
--------------------------------------------------------
Without `--agent-mode` the server refuses to emit tool calls at all ("Running
shell commands requires Agent mode"), so every local-search node fails on the
permission gate rather than on routing. That measures the wrong thing. The
runner now sets FORMAL_AI_AGENT_MODE=1 and advertises a bash+websearch tool set
matching what opencode sends. Anyone re-running this must keep both.

The websearch tool is deliberately NOT executed by the harness (no live network
in a reproducible measurement). Nodes that route to websearch therefore fail on
missing content — which is the correct outcome, since routing to the web for a
local-filesystem request is itself the defect.

CI SAFETY
---------
`experiments/` is excluded from `any-code-changed` (scripts/detect-code-changes.rs:155)
and from the release-gating diff (.github/workflows/release.yml:381). Keep this
directory free of *.md and *.mjs: `docs-changed` and `mjs-changed` are computed
by file extension alone and ignore the folder exclusion. That is why this file
is README.txt and not README.md.
