TASK (tag DEBUG3): finish R383 (docs/requirements/issue-0538-detailed-meanings-and-words.md row R383; protocol in docs/vscode/debugger.md).

Done already:
- the step-through server, protocol and token (round 12);
- per-turn Mermaid source and the routed method's Rust/JS locations, the SVG pane, and the held turn (round 13, DEBUG2);
- the browser end-to-end test `rust/tests/e2e/tests/issue-667-debugger-view.spec.js` and the live capture `docs/case-studies/issue-667/debugger-live-session.png` (round 14, LEAD).

Remaining, in order:

1. **Per-stage source locations.** Every stage of a turn now names the turn's routed method (`MethodRegistry::method_for_route`), not the code that emitted that stage.
   - Give each stage the Rust and JS location of the code that emits its step (`impulse`, `detect_language`, `formalize`, `compute`, `compute_engine`, `dispatch_handler`, `rule_verification`, `deformalize`, …).
   - Derive the locations from the source rather than a hand-kept table. For example, a generator script writes `data/meta/debug-stage-sources.lino` from the definitions that push each step name; a gate checks that it is current, the way `scripts/generate-system-diagrams.mjs --check` works.
   - `js/server/debug-stage.mjs` and `rust/src/server/debug_stage.rs` read it. Keep the routed method as a separate field.
   - Extend `rust/tests/web/server-debug-session.test.mjs` and `rust/tests/unit/specification/debug_session.rs` with the exact per-stage locations of the `What is 2 + 2?` turn.
2. **Suspend between stages where the solver is staged.**
   - Today the solved turn is held and revealed stage by stage.
   - Where the JS solver already runs in steps (the agentic planner's tool loop in `js/server/solve.mjs`, the worker's staged pipeline), await the session gate between steps, so a held turn has not computed its later stages yet.
   - Document exactly which stages are truly suspended and which are revealed. Mirror the change in Rust where the same staging exists.
3. Extend the e2e spec to assert that the source panes change from stage to stage once item 1 lands.

Update row R383 honestly: "implemented" only when stepping, all five panes and per-stage locations are pinned by tests and the suspension is documented. Ledger rows T140–T149. The local web build needs the bun in `.bun-version`; see the README's routine commands.
