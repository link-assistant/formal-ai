Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E54 in `proposed-issues.md` there; added by PR #652).


**Problem**

The meta-language self-representation (`data/meta/self-ast.lino`, R381) is
still pinned to a single module (`src/agentic_coding/planner.rs`). PR #601
itself named "the smallest real next slice: extend the pinned target … to a
directory census", and while PR #637 widened the *source-links* index, the
CST/AST census never grew. Self-coding (E35/E36) needs the system to see its
own code: a planner that can only introspect one file cannot plan edits
across the workspace.

**Approach**

1. Generalize the census recipe to take a target set (directory glob) and
   emit per-module `.lino` census documents plus a workspace index —
   incremental, so one changed module re-censuses one document.
2. Scale honestly: full-fidelity AST for `src/agentic_coding/` first, then
   signature-level census (items, symbols, spans) for the rest of `src/`,
   with a documented fidelity marker per module — avoiding a multi-megabyte
   seed while keeping every module addressable.
3. Wire freshness into CI: a check that fails when a committed census
   diverges from the source it describes (the drift-guard pattern used by
   the method registry from #559).
4. Feed E35: the general planner resolves edit targets through the census
   index instead of hardcoded paths.

**Existing components**

- `data/meta/self-ast.lino` + its census recipe and pinning tests.
- `src/agentic_coding/{self_source,source_graph}.rs` widened by PR #637.
- The #559 method registry drift-guard.

**Acceptance criteria**

- `data/meta/self-ast/` contains a census for every `src/` module with its
  fidelity marker; the workspace index resolves any `path:symbol` the
  method registry knows.
- `cargo test self_ast_census` — census regenerates deterministically;
  drift check fails on a fixture with a stale census.
- A planner test resolves an edit target in a module other than
  `planner.rs` via the census index.

