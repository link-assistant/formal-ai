# Issue #842: a strict, growing task-quality ratchet

Issue #842 asked for the 24-node task ladder from #840 to become an executable
quality program: raise the 8/24 baseline, remove the inverted L1/L4 result,
enforce route parity, reject refusals and capability menus, run in CI, and grow
with later reports.

## Starting point

The production grounded-action recipe merged from #840 already solved the
major composition problem and its earlier harness reported 24/24. Re-judging
the same responses with stronger assertions exposed two false greens:

| Node | Previous response/route | Why it was not a pass |
| --- | --- | --- |
| `827.L4.a` | generic Russian “could not determine” fallback | the node had no positive answer assertion |
| `827.L4.b` | `websearch` for a contextual pronoun | the node had no route assertion or required clarification |

Regression tests were added before the fix. The first failed because the
Russian example request had no plan; the second failed because the isolated
pronoun question emitted a web-search tool call.

## General production rule

Definition-example requests are represented as one slotted meaning role.
English and Russian use prefix slots, Hindi uses a suffix slot, and Chinese uses
a circumfix slot. The planner extracts the subject from slot structure, so a
new language or phrasing is seed data rather than a prompt-specific Rust branch.

A contextual word-meaning query is also resolved structurally. When the parsed
subject is a non-referential term and the query only supplies a sentence—not an
earlier antecedent—the planner returns the existing seeded clarification. It
does not search the public web for a pronoun.

The focused regression test covers both rules in
`tests/unit/issue_842.rs`, including all four supported language families.

## A falsifiable ladder

`experiments/issue_840_task_ladder/ladder.py` separates:

- assistant output, which alone may satisfy answer assertions;
- raw tool results, retained only as transcript evidence;
- required and forbidden tool routes;
- forbidden command fragments;
- execution errors, exact generic refusals, and capability-menu fallbacks.

The offline web corpus contains realistic adjacent headings and page controls.
Those strings are forbidden in answer assertions, so a search result cannot be
copied wholesale and called synthesis.

The baseline comparison is per stable task ID. It rejects a removed node, a
regressed passing node, or any failing appended node. New green nodes are
allowed, which makes “add every new report” an executable extension protocol
instead of a fixed aggregate score.

The current strict measurement is recorded in
`experiments/issue_840_task_ladder/results.json`: 24/24 overall, with both L1
and L4 at 100%.

## Failure-derived learning without silent promotion

Each run also writes a learning-proposal artifact. Only observed failures become
candidates, and each candidate retains the stable ID, prompt, assistant output,
tools, commands, error, and violated assertions. The decision is always
`awaiting_human_review`. The proposal cannot change solver behavior or advance
the baseline until every task passes and a human approves it.

This connects new reports to Formal AI's existing review-gated learning model
without teaching the solver from its own unverified output.

## Formal AI and the real Agent CLI

The release workflow already drives the whole grounded-action journey through a
real Agent CLI and a representative node through the OpenCode TUI. This change
also asks Formal AI, through the real Agent CLI, to author one reviewed smallest
leaf: the durable stable-ID ratchet invariant.

The replay driver is `experiments/issue_842_self_authoring/run.sh`; its raw
Agent stream, Formal AI trace, and byte-identical generated artifact are in
`self-hosting/`.

The differential self-hosting gate also measured this branch's preserved
pre-merge history. Rather than attribute human commits to Formal AI, a second
real Agent CLI session (`ses_052baa382ffe4lQ4ABBJC6806D`) ran the existing
self-AST recipe. The same deterministic CST/AST engine then expanded the live
slice to all 330 owned-source documents. `self-hosting-census/` preserves the
session, the focused artifact, and the complete 13,160-line Links Notation
census. This gives a failed ladder node an auditable map from its routing or
synthesis observation to the source surface that a reviewed repair would
change.

After the final explicit-evidence planner correction changed the owned source,
session `ses_03ab2b5ecffeE6OM87CFgu5dJ2` reran that self-AST axis through the
real Agent CLI. A separate session, `ses_03990d3dcffe80WtbvI6wSQ1O0`, reran the
self-healing axis whose repair-case source map also depends on the planner.
`self-hosting-census-refresh/` retains both final-source streams, Formal AI
traces, focused artifacts, and the census renderer summary. The historical
workspace snapshot and legacy planner-derived fixtures were generated from that
same source state. Its ratchet checks that the recorded census index still
resolves the recorded module by content ID, without rewriting history when a
later change adds source modules. The independent whole-workspace census tests
regenerate every current document from live source and fail byte-for-byte on
present-source drift.
