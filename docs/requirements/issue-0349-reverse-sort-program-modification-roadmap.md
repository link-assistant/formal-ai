## Issue #349 Reverse-Sort Program Modification Roadmap

Issue [#349](https://github.com/link-assistant/formal-ai/issues/349) reported a
multi-turn Russian coding dialog where the assistant wrote a Rust program,
successfully modified it to accept a path argument, then answered the next
program-modification follow-up with `intent: unknown` instead of reversing the
sort order. The roadmap split that defect into issues
[#355](https://github.com/link-assistant/formal-ai/issues/355) through
[#365](https://github.com/link-assistant/formal-ai/issues/365). The final epic
[#365](https://github.com/link-assistant/formal-ai/issues/365) records that
issues #355-#364 are closed and merged, and that the #349 dialog is green in the
Rust core, browser worker, diagnostics surface, and coding-modification
benchmark.

| ID | Requirement | Status |
| --- | --- | --- |
| R256 | The issue #349 reverse-sort follow-up must route to a real program-modification answer instead of `unknown`. | Implemented by `tests/integration/issue_349_reverse_sort.rs::issue_349_reverse_sort_follow_up_must_not_be_unknown`: the fifth turn routes to `write_program`, preserves the path-argument edit, and emits reverse-sort code. |
| R257 | Bare program-result follow-ups must bind to the active program artifact before program lowering. | Implemented by the active-program coreference flow added for #357 and covered by `tests/unit/specification/code_generation_coreference.rs`; the #349 trace records `referent=active_program_artifact task=list_files_arg language=rust`. |
| R258 | Program modifiers must compose from data instead of a one-off `path_argument` allowlist. | Implemented by the generalized modifier model in `src/program_plan.rs`, `src/rule_synthesis.rs`, `data/seed/operation-vocabulary.lino`, and `data/seed/program-plan-rules.lino`; tests cover `path_argument` plus `reverse_sort` composition into `list_files_arg_reverse_sort`. |
| R259 | Unknown program follow-ups must attempt reasoned rule construction before reporting failure. | Implemented by the #359 rule-construction path and covered across supported prompt languages by `tests/unit/specification/code_generation_program_modifiers.rs` and the #349 diagnostics test. |
| R260 | The write-program reasoning chain must be inspectable and diagnostics must remain default off. | Implemented by `SolverConfig::diagnostic_mode`, `tests/integration/issue_349_reverse_sort.rs::issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain`, and browser e2e coverage in `tests/e2e/tests/issue-360.spec.js`. |
| R261 | The #349 behavior must be consistent across the Rust core and browser worker. | Implemented by `experiments/issue-361-cross-runtime-parity.mjs`, which verifies the browser-worker mirror handles the same turn-5 follow-up, trace, and no-active-program guard as the Rust core. |
| R262 | The #349 dialog must be part of a multilingual coding-modification benchmark ratchet. | Implemented by `data/benchmarks/coding-modification-suite.lino` and `tests/unit/specification/coding_modification_benchmarks.rs::issue_362_multilingual_multi_turn_coding_modification_ratchet`. |
| R263 | Resolvable program modifications must not push the user toward a response-level report action. | Implemented by `tests/e2e/tests/issue-363.spec.js`, which verifies the resolved reverse-sort follow-up has no message-level report link. |
| R264 | Unknown traces must feed a white-box self-improvement loop gated by the benchmark. | Implemented by `docs/design/self-improvement-loop.md`, `src/self_improvement.rs`, and `tests/unit/specification/self_improvement.rs`. |
| R265 | The issue #349 child issues and their final status must remain auditable from repository documentation. | Implemented by `docs/case-studies/issue-365/README.md` and the Issue #349 section in `ROADMAP.md`, which map #355-#365 to their closing PRs and verification evidence. |
