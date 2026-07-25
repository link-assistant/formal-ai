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
                  appear), `expect_any` (at least one must appear), `forbid`
                  (none may appear), `expect_tool` / `forbid_tool` (which tool
                  the node must / must not be routed to) and `command_forbid`
                  (substrings the emitted shell command may not contain), plus a
                  `note` saying which standard from #840 it exercises.
  web_fixtures.json  Offline corpus the harness serves to `websearch` calls.
                  Keyword-matched documents, deliberately including page
                  furniture ("развернуть"), so a node that echoes the page
                  instead of synthesising an answer is visible as a `leaked`
                  failure rather than a pass.
  run_ladder.sh   Boots the server with agent mode, runs a real agentic loop
                  (executes returned bash commands in a sandbox and feeds the
                  output back, as opencode does), writes results.json.
  results.json    Output of the last run. Regenerate; do not hand-edit.

USAGE
-----
  cargo build --release
  experiments/issue_840_task_ladder/run_ladder.sh
  ONLY=838 experiments/issue_840_task_ladder/run_ladder.sh   # filter by id

Knobs: BIN, PORT, TASKS, OUT, ONLY, SANDBOX, SANDBOX_KEEP, FIXTURES, BASELINE.

A node fails if any expected substring is missing, any forbidden one appears,
it refuses, it answers with the capability menu, it is routed to a forbidden
tool, or its shell command contains a forbidden fragment.

With BASELINE pointing at a previous results.json the runner exits 1 when the
score drops. That is how the `task-ladder` job in .github/workflows/release.yml
turns this measurement into a regression gate.

BASELINE @ v0.303.0 (main 1873e873), 2026-07-25
-----------------------------------------------
  TOTAL  8/24
  by level:  L1 1/3   L2 1/6   L3 2/7   L4 4/8
  by seed:   #838 3/10   #827 1/7   #826 4/7

Note the inversion: the system did BETTER on the smallest subtasks (L4 4/8)
than on the original user-facing requests (L1 1/3). Decomposition was not the
weak point; composing a whole answer was.

MEASURED @ issue #842 (branch issue-842-09f1f0f99558)
-----------------------------------------------------
  TOTAL  16/24
  by level:  L1 2/3   L2 3/6   L3 6/7   L4 5/8
  by seed:   #838 7/10   #827 2/7   #826 7/7

The inversion is gone: L1 rose from 1/3 to 2/3 (+33pp) while L4 rose from 4/8
to 5/8 (+12pp), so the whole-answer rungs improved at least as fast as the
atomic ones. Note that the judging also got STRICTER between the two runs
(refusals, capability-menu answers, tool misroutes, forbidden command
fragments and leaked page furniture are all failures now), so 16/24 is not
measured on an easier scale than 8/24.

The headline routing asymmetry is closed. All three phrasings now route to
bash and pass:

  838.L1    "Find hive-mind-control center folder on my desktop"   PASS
  838.L3.a  "Search hive-mind-control-center on my desktop"        PASS
  838.L3.b  "Find hive-mind-control center folder on desktop"      PASS

#826 is 7/7: "ФБС vs ФБО" no longer refuses, and no node anywhere answers
with the capability menu.

WHAT STILL FAILS (8 nodes) — read this before claiming the ladder is done
-------------------------------------------------------------------------
  838.L2.a  A question ABOUT a request ("Is the request '...' a local
            filesystem search or a web search?") is executed as one instead of
            answered. Meta-questions are not distinguished from requests.
  838.L4.b  "Is there a folder named exactly '...'" does not report the
            near-miss sibling name that exists on the fixture desktop.
  838.L4.c  "is X a file or a folder?" routes to websearch, not bash.
  827.L1    Routes to the web correctly but pastes the page back, page
            furniture included, instead of synthesising a definition.
  827.L2.a  "Дай определение слова X одним предложением" now extracts the
            concept (see data/seed/prompt-patterns.lino) but an unresolved
            concept does not escalate to the advertised websearch tool, so it
            refuses.
  827.L3.a  Same escalation gap, plus the subject keeps the trailing
            instruction: "a fufloмицин (фуфломицин)? Answer in English".
  827.L4.a  "Приведи один пример препарата, который называют X" — an example
            request, not a definition request; unrecognised.
  827.L2.b  Two sentences in one turn: only the coreference half is answered.

The common root cause under #827 is one thing: a concept lookup that fails
locally does not escalate to the advertised search tool. That is the next
measurable step, not a phrasing fix.

A FIRST RUN WITHOUT AGENT MODE SCORED 4/24 AND WAS WRONG
--------------------------------------------------------
Without `--agent-mode` the server refuses to emit tool calls at all ("Running
shell commands requires Agent mode"), so every local-search node fails on the
permission gate rather than on routing. That measures the wrong thing. The
runner now sets FORMAL_AI_AGENT_MODE=1 and advertises a bash+websearch tool set
matching what opencode sends. Anyone re-running this must keep both.

The websearch tool is never given live network access — a measurement that
depends on the internet is not reproducible. It is answered from
web_fixtures.json instead, a small keyword-matched corpus. That is what makes
the #827 nodes measure SYNTHESIS: the fixture text is known, so an answer that
merely echoes it (page furniture and all) is distinguishable from an answer
that composes a definition. Nodes that route to the web for a local-filesystem
request still fail, on `forbid_tool` — that misroute is itself the defect.

CI SAFETY
---------
The `task-ladder` job in .github/workflows/release.yml runs this harness with
BASELINE=results.json on every code change, prints the score to the step
summary and uploads the results as an artifact. Editing files in this directory
does not by itself trigger that job (see the exclusion below); a change to the
solver does. That is what makes a regression in the score visible.

`experiments/` is excluded from `any-code-changed` (scripts/detect-code-changes.rs:155)
and from the release-gating diff (.github/workflows/release.yml:381). Keep this
directory free of *.md and *.mjs: `docs-changed` and `mjs-changed` are computed
by file extension alone and ignore the folder exclusion. That is why this file
is README.txt and not README.md.
