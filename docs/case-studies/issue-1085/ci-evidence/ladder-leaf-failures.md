# What the compile-and-test ladder measured: 15 of 32 leaves

Issue #1085 (D4). Run 34206203615 on branch `issue-1085-9c2f4e7a1b3d`,
2026-09-08, `TREE_DEPTH=5`: 32 leaves selected, 17 failed. Each leaf asks for
one change to one tracked Rust file (a member insertion, a literal
replacement or an identifier rename) and passes only when the worktree shows
exactly that file modified, the marker absent from `HEAD`, `cargo check --lib`
compiling, and `cargo test --test unit <module>` passing.

The seventeen failures are not seventeen problems. They are three mechanisms,
read from `formal-ai.log` and `agent-stream.jsonl` in the run's tree evidence
artifact.

## A. The continuation turn is answered by web search (8 leaves)

L01, L02, L04, L05, L06, L08, L09, L10.

The Agent CLI sends `Continue if you have next steps` after a tool result.
Formal AI routes that prompt to the web-search handler, which fetches
unrelated pages (`docs.aws.amazon.com/lexv2/.../paths-nextstep.html`,
`docs.continue.dev/cli/quickstart`) and answers with their prose. The session
then ends on that answer instead of the effect document. L05 made 672
web-fetch and web-search calls in one node; L01 and L02 ended on
`Request failed with status code: 403` from a rate-limited fetch, and L04 on
Exa's free-tier limit.

The prompt carries no request. A turn whose whole content is a continuation
cue should resume the task in the session, never open a search.

## B. An edit is verified against a file the planner generated (7 leaves)

L15, L17, L18, L22, L24, L26, L28.

`plan_generated_source_step` in `src/agentic_coding/code_task.rs` claims the
leaf task, because "keep it valid Rust" reads as a program request. It builds
its own artifact for the task, has the agent write the file, then `cat`s the
path and compares the bytes with that generated artifact. The agent had
applied the requested replacement to the real file, so the bytes differ from
the artifact and the run reports
`Verification failed for <path>: the observed bytes differ from the planned
workspace effect` (`data/seed/multilingual-responses-agentic-tools.lino`).

A task that names an existing tracked file and a replacement is a
modification, not a generation. Verifying it against a synthesised file can
only fail.

## C. The change is reported without being made (2 leaves)

L14, L16.

The session ends on `result=Replaced <old> with <new> in <path> and observed
the result.` with no modification in the worktree. The read-and-echo criteria
of the 2026-08-28 run accepted exactly this; the compile-and-test criteria are
what make it visible.

## Why the record is 15 and not 32

`data/meta/ladder-ratchet.lino` records `leaf_nodes_passing 15`. The workflow
fails a full-width run that passes fewer and prints a notice when it passes
more. A record of 32 would have been a number no run has ever produced, and
the earlier `deepest_passing_level 5` was carried over from the read-and-echo
baseline. The ratchet also had its comparison inverted: it errored when the
measured level was *deeper* than the record. Both are fixed in this pull
request.

Mechanisms A and B are behaviour defects in Formal AI itself, not in the
ladder: #1095 (E113) and #1096 (E114), sub-issues of #1085 and blocked by it.
Mechanism C is what the compile-and-test criteria were introduced to catch and
needs no separate issue.
