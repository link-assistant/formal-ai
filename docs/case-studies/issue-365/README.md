# Issue #365 Closure Report

Issue [#365](https://github.com/link-assistant/formal-ai/issues/365) is the
epic that closes the issue #349 reverse-sort roadmap after its child issues
[#355](https://github.com/link-assistant/formal-ai/issues/355) through
[#364](https://github.com/link-assistant/formal-ai/issues/364) are done.

The epic is now ready to close: every child issue is closed, every child PR is
merged into `main`, and the original issue #349 dialog is covered by active
Rust, browser-worker, diagnostics, and benchmark tests.

## Child Issue Status

| Issue | Closing PR | Result |
| --- | --- | --- |
| [#355](https://github.com/link-assistant/formal-ai/issues/355) | [#366](https://github.com/link-assistant/formal-ai/pull/366) | Added the issue #349 reverse-sort reproduction as integration coverage. |
| [#356](https://github.com/link-assistant/formal-ai/issues/356) | [#367](https://github.com/link-assistant/formal-ai/pull/367) | Documented rule synthesis over Links Notation in `docs/design/rule-synthesis.md`. |
| [#357](https://github.com/link-assistant/formal-ai/issues/357) | [#369](https://github.com/link-assistant/formal-ai/pull/369) | Bound bare program-result follow-ups to the active program artifact. |
| [#358](https://github.com/link-assistant/formal-ai/issues/358) | [#370](https://github.com/link-assistant/formal-ai/pull/370) | Generalized composable program modifiers, including `reverse_sort`. |
| [#359](https://github.com/link-assistant/formal-ai/issues/359) | [#371](https://github.com/link-assistant/formal-ai/pull/371) | Added reasoned rule construction for unknown program follow-ups. |
| [#360](https://github.com/link-assistant/formal-ai/issues/360) | [#372](https://github.com/link-assistant/formal-ai/pull/372) | Exposed the full write-program diagnostics chain, default off. |
| [#361](https://github.com/link-assistant/formal-ai/issues/361) | [#373](https://github.com/link-assistant/formal-ai/pull/373) | Added Rust/browser-worker parity coverage for the #349 follow-up. |
| [#362](https://github.com/link-assistant/formal-ai/issues/362) | [#374](https://github.com/link-assistant/formal-ai/pull/374) | Added the multilingual coding-modification benchmark and ratchet. |
| [#363](https://github.com/link-assistant/formal-ai/issues/363) | [#375](https://github.com/link-assistant/formal-ai/pull/375) | Made "Report issue" a last resort for resolvable reverse-sort edits. |
| [#364](https://github.com/link-assistant/formal-ai/issues/364) | [#376](https://github.com/link-assistant/formal-ai/pull/376) | Added the white-box unknown-trace self-improvement loop. |

## Requirement Mapping

| Requirement from issue #349 | Closure evidence |
| --- | --- |
| R1: the reverse-sort follow-up must not return `unknown`. | `tests/integration/issue_349_reverse_sort.rs::issue_349_reverse_sort_follow_up_must_not_be_unknown`. |
| R2: reason over Links Notation instead of memorizing one-off rules. | `docs/design/rule-synthesis.md`, `src/rule_synthesis.rs`, `src/program_plan.rs`, and rule-construction tests. |
| R3: keep the reasoning white-box. | Diagnostics traces include route attempts, coreference binding, modifier detection, rule construction, verification, and the final program plan. |
| R4: expose verbose diagnostics with default-off behavior. | `SolverConfig::diagnostic_mode`, `tests/integration/issue_349_reverse_sort.rs::issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain`, and `tests/e2e/tests/issue-360.spec.js`. |
| R5: fix every runtime surface. | `experiments/issue-361-cross-runtime-parity.mjs` verifies the browser worker mirrors the Rust core for the #349 flow. |
| R6: add a bulk multilingual coding-modification suite. | `data/benchmarks/coding-modification-suite.lino` and `tests/unit/specification/coding_modification_benchmarks.rs`. |
| R7: reduce report-pressure for resolvable prompts. | `tests/e2e/tests/issue-363.spec.js` checks the resolved reverse-sort follow-up has no response-level report action. |
| R8: add white-box self-improvement from unknown traces. | `docs/design/self-improvement-loop.md`, `src/self_improvement.rs`, and `tests/unit/specification/self_improvement.rs`. |
| R9: plan the work as dependency-linked GitHub issues. | Issues #355-#365 were created and wired; this report records the final completion state. |
| R10: preserve the deep case study and evidence. | PR [#350](https://github.com/link-assistant/formal-ai/pull/350) contains the original case study, raw issue/PR data, reproduction script, and prior failed-session log. |
| R11: report external issues where needed. | No external project owned the defect; the fix was local to this repository. |
| R12: recover from the failed prior session. | The prior failed-session log is preserved and summarized in the issue #349 case study. |

## Verification Contract

The epic is considered green when these checks pass:

```sh
cargo test --test integration issue_349 -- --nocapture
node experiments/issue-361-cross-runtime-parity.mjs
cargo test --test unit specification::coding_modification_benchmarks::issue_362_multilingual_multi_turn_coding_modification_ratchet -- --nocapture
cargo test --test unit specification::self_improvement -- --nocapture
```

Together these checks prove the original Russian dialog routes to
`write_program`, keeps the path-argument edit, reverses the sort order, exposes
the diagnostic reasoning chain, matches the browser-worker runtime, remains in
the benchmark ratchet, and feeds the white-box self-improvement policy.
