## Issue #923 Symbolic-Kernel Coverage Growth

Issue [#923](https://github.com/link-assistant/formal-ai/issues/923) implements
E76 from the issue #914 planning batch. PR
[#1006](https://github.com/link-assistant/formal-ai/pull/1006) adds two bounded,
non-neural reasoning capabilities and measures both against pinned upstream
Rust examples. See `docs/case-studies/issue-923/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R923-1 | Add at least two general symbolic reasoning capabilities beyond propositional SAT and linear arithmetic. | `decision/equality.rs` performs bounded e-graph saturation over generic symbolic S-expressions; `decision/rules.rs` evaluates bounded, function-free positive Datalog to its least fixed point. |
| R923-2 | Exercise external reasoning cases and record honest scores under `data/benchmarks/`. | The #698 harness mechanically adapts the first 20 unconditional laws from pinned egg `tests/math.rs` and all five asserted consequences from Ascent's pinned transitive-closure example. The committed scores are 20/20 egg laws and 5/5 Ascent closure assertions. |
| R923-3 | Keep neural inference out; license-check and feature-gate every new dependency. | The only new dependency is MIT-licensed `egg` 0.11.0, optional with default features disabled and exposed through `equality-saturation`; Datalog is implemented in-tree without another runtime dependency. |
| R923-4 | Preserve the existing reasoning regression floor and make solver limits honest. | No pre-existing reasoning case is removed or relaxed; focused regressions assert proof, inference, and sound failure behavior. Equality search failure, unsafe Datalog rules, and Datalog resource exhaustion are inconclusive, never false disproofs. |
| R923-5 | Preserve reproducible issue, PR, upstream, benchmark, and self-hosting evidence. | The issue and PR case studies retain GitHub snapshots, source/license research, exact commands and scores, and real Agent CLI session `ses_001f733ceffe5UboLW4JATfkoZ`. |
