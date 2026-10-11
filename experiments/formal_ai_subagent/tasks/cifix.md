TASK (tag CIFIX): make the Rust test suite green for the next push without building Rust locally.

Input: `experiments/formal_ai_subagent/sandboxes/ci-a75bf3772-rust-failures.txt` holds the panic output of the 19 Rust tests that failed in CI on a75bf3772 (CI/CD Pipeline run 37738783201, shards 1-4; re-fetch any detail with `gh run view 37738783201 --log-failed`). The working tree has moved on since then (round 14 changes from TEACH-C/D, MIGRATE3, ROUTE2, DEBUG3, REPO-PROTO and LEAD), so for each test first decide whether it still fails on the current tree.

1. Classify every failure:
   - (a) census or generated-artifact drift, which LEAD fixes with the census commit (`issue_538_agentic::committed_self_ast_*`, `issue_558_self_healing::*`);
   - (b) a stale expectation after an intended change;
   - (c) a real regression;
   - (d) a ladder or requirement-resolution test reading a moved or renamed file (`issue_1069_*`, `issue_1085_requirement_resolution::*`, `issue_1066_agent_ladder::*`).
2. Fix (b), (c) and (d) at the source:
   - reproduce through the JS twin when one exists (node, never cargo);
   - correct the Rust by reading it;
   - keep both roots in step.

   Examples to look at:
   - `data_files::meaning_slugs_are_globally_unique` and `seed_lino_values_never_pipe_pack_multi_values` point at seed data;
   - `issue_918::coding_path_has_complete_metadata…` at seed metadata;
   - `issue_1138_surface_honesty::guidance_has_a_spanish_branch` at a missing es surface;
   - `issue_1175_routing_probes::the_native_solver_answers_every_probe_as_measured` prints the corrected `rust_misroute` line in its panic.
3. Re-check every item in the "risks" lists below against the current tree.
   - The E2E local-web job failed in diagnostics mode because `js/app/debugger-view.jsx` did not bind `h`. That is fixed in the tree; confirm it by running `rust/tests/e2e/tests/demo.spec.js -g "diagnostics toggle"` with the bun from `.bun-version` (one is unpacked under `experiments/formal_ai_subagent/sandboxes/bun-1.4.0/`).
   - Look at the Coverage run 37738782962 failures too (`gh run view 37738782962 --log-failed`), and say whether they share causes with the test shards.

Rules: never run cargo or any script that calls it. `local-gates.mjs` is the gate runner. Claims go in claims.md. Report every test with its class and fix, and list anything that only CI can confirm. Ledger rows T180–T189 are for any Formal AI tasks you delegate.
