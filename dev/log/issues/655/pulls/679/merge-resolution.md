# Merge conflict resolution — issue #655 / PR #679

Date: 2026-07-13

## Context

`origin/main` advanced past the PR #679 branch point (notably PR #678 for
issue #676, which split shell-command and code-translation helpers out of
`src/agentic_coding/planner.rs`). Merging `origin/main` into
`issue-655-352ae09c0c14` produced four conflicts.

## Conflicting files and how they were resolved

1. `.github/workflows/release.yml`
   - Conflict was purely additive: the branch added a new E2E step
     "Replay Hive Mind self-coding inner loop (issue #655)".
   - Resolution: keep the new step (HEAD), drop the empty `main` side.

2. `data/meta/self-ast.lino`
3. `data/meta/self-healing-case.lino`
4. `docs/case-studies/issue-538/agent-cli-session-self-ast.json`

   These three are *generated* artifacts: an abstract-syntax node census of
   `src/agentic_coding/planner.rs`, produced deterministically by
   `render_ast_document` / `render_document`. The branch's committed census
   (`total_link_count 20672`, `named_node_count 4158`) was generated against
   the pre-#676 planner. After the merge, `planner.rs` is byte-for-byte equal
   to `origin/main`'s planner (`git diff origin/main -- planner.rs` = empty),
   so the correct census is `origin/main`'s (`total_link_count 19169`,
   `named_node_count 3880`).

   Resolution: `git checkout --theirs` for all three, i.e. take the `main`
   census.

## Verification

Re-generated census assertions pass against the merged source:

- `cargo test --test unit self_ast` — 10 passed
- `cargo test --test unit self_coding` — 2 passed
  (`self_coding_session_replays`, `self_coding_capture_contains_every_layer`)
- `cargo test --test unit self_healing` — 7 passed
  (incl. `committed_self_healing_case_is_generated_and_written_by_the_driver`)

The census tests re-render the document from the merged `planner.rs` and
compare byte-for-byte to the committed artifacts, confirming the `--theirs`
resolution is exactly what the merged source produces.
