## Issue #327 Cross-Runtime Synthesis Parity

Issue [#327](https://github.com/link-assistant/formal-ai/issues/327)
requires the browser worker to derive the same synthesis, numeric, program,
and text answers as the Rust core for the E28-E31 work, while keeping Rust/WASM
as the owner of shared primitives and JavaScript as browser glue.

| ID | Requirement | Status |
| --- | --- | --- |
| R246 | Browser-worker synthesis prompts must route to typed derived intents instead of `unknown` or legacy write-program templates. | Implemented by `tryLinkNativeSynthesis`, `tryProgramSynthesis`, and `tryTextManipulation` in `src/web/formal_ai_worker.js`, inserted ahead of the legacy arithmetic/write-program fallbacks. |
| R247 | Rust and browser parity must be checked from a shared fixture that covers algebra substitution, renumbered remainder-sale arithmetic, object counting with a distractor, unseen Python synthesis, and chained text manipulation. | Covered by `data/parity/cross-runtime-synthesis.json`, `shared_cross_runtime_synthesis_fixture_matches_rust_solver`, and `tests/e2e/tests/issue-327.spec.js`. |
| R248 | Anti-memorization checks must hold on the browser side for renumbered numeric answers, distractor counts, and unseen synthesized programs. | The shared fixture includes forbidden fragments such as `18`, `4`, and `legacy_intent`; the browser e2e asserts they do not appear in rendered answers and checks semantic forbidden evidence for legacy routes. |
| R249 | WASM must remain the bridge for shared primitives while JS performs browser-only composition and glue. | Browser synthesis delegates arithmetic evaluation and stable id creation through `wasmEvaluateArithmetic` and `wasmStableId` when available, with JS fallbacks only for no-WASM compatibility. |
