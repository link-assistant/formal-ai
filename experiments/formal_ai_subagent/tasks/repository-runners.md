TASK (tag REPO-RUNNERS): give the repository workspace protocol's runners a JavaScript twin (R1138-3-5, R1188-U29 JavaScript first).

The measure:
- R1138-3-5 in `docs/requirements/issue-1138-repository-workspace-protocol.md` is partial. Rust runs the one protocol document (`data/meta/repository-workspace-protocol.lino`) for three callers: SWE-bench, the coding ladder and self-authoring.
- `js/agentic/crate/repository_workspace.mjs` (127 lines) ports only the document parser, the editor rule and the trace renderer. The runners have no JavaScript twin:
  - `WorkspaceProtocol::execute` in `rust/src/repository_workspace/mod.rs`, with its stages in `clone.rs`, `locate.rs`, `edit.rs`, `verify.rs`, `diff.rs` and `outcome.rs`;
  - `authoring_loop::run_authoring_with` in `rust/src/authoring_loop.rs`.

The JavaScript twins the runners need already exist:
- `js/agentic/crate/execution_evidence.mjs` (`Evidence::observed`);
- `meta_frame.mjs` and `obligation_ledger.mjs` (`NeedLedger`, `need_ledger_with_execution`);
- `engine_stable_id.mjs`, `needs.mjs` and `self_ast_census.mjs` (`WorkspaceCensus`);
- `js/agentic/requirement_resolution.mjs` and `js/agentic/structured_edit.mjs`.
Processes and files go through an injected `io`, as `nodeHistoryIo` in `js/agentic/crate/history_store.mjs` does. The allowlist is `data/seed/repository-command-allowlist.lino`.

Work progressively, the whole first and then sharper:
1. **The stage loop.** Port `execute` as one loop over the document's stages, each stage a function that reads the workspace and appends to the outcome: `clone` checks the observed head, `locate`, `read`, `edit`, `verify`, `diff`, `commit`.
   - Processes (`git rev-parse`, `git diff`, the verify command) go through one injected runner, so the tests can use a fake one, as `rust/tests/unit/ci-cd/protocol_callers.rs` does with fake executables.
   - Templates come from the same seed (`render_protocol_template`).
   - Split the twin into several modules under `js/agentic/crate/` that mirror the Rust files. Keep each file ≤1500 lines.
2. **The authoring loop.** Port `run_authoring_with` to iterate the same document's stages. The `serve` and `session` stages (editor `agent_session`) are data handed to the caller, not run by the loop.
3. **The trace.** Both runners write the same `repository-protocol.lino` through the existing trace renderer, one stage line per declared stage with its status.
4. **Pins.** Add `rust/tests/web/repository-workspace-runners.test.mjs`, which replays the Rust cases of `protocol_callers.rs` and `rust/tests/unit/specification/repository_workspace_protocol.rs` on a temporary git repository with a fake runner. It must assert:
   - the identical stage names of the three callers;
   - that no server is spawned without the serve stage;
   - the opened commit gate in the trace;
   - the base-commit mismatch stop and its open item.
5. **Rows.** Update R1138-3-5 with the runners' twin and its test. Then run the requirement pipeline and `node scripts/render-progressive-plan.mjs --write`.

Rules: the same as `tasks/generalize.md`.
- No cargo, rustc or rust-script locally. The Rust side should not need to change.
- After JS edits, run `node scripts/translate-es.mjs --write`.
- New js/agentic/crate modules: run `node scripts/generate-worker-crate-modules.mjs --write` if the worker loads them, and name them to LEAD for re-projection by `scripts/translate-js-rust.mjs`.
- Claim your files in `claims.md`.
- Use Formal AI from JavaScript (`experiments/js_dogfood/drive.mjs`) for single-line edits, and log them as T870-T899.
- Don't commit. LEAD integrates.
