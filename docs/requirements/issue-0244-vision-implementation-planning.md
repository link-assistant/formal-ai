## Issue #244 Vision Implementation Planning

Issue [#244](https://github.com/link-assistant/formal-ai/issues/244) is a
meta-planning issue: before any new feature code, it requires the documentation
to **fully track implementation progress** and stay **in sync with the actual
state of the code**, and then for the GitHub issues that fully implement the
vision to be created from a deep case study (with online research, every
requirement listed, and a solution plan per requirement that checks existing
components). The deliverables of this requirement are documentation and planning
artifacts, not feature code; the feature work itself is tracked by the created
planning issues. The first foundation batch (E1-E14, [#246](https://github.com/link-assistant/formal-ai/issues/246)-[#259](https://github.com/link-assistant/formal-ai/issues/259))
and the second follow-up batch (E15-E20, [#278](https://github.com/link-assistant/formal-ai/issues/278)-[#283](https://github.com/link-assistant/formal-ai/issues/283))
are all closed and merged. A third audit (2026-05-26) re-read the closed issues
against the vision and opened the reasoning batch E21-E27 ([#298](https://github.com/link-assistant/formal-ai/issues/298)-[#304](https://github.com/link-assistant/formal-ai/issues/304)):
reasoning under unknowns, intent formalization in Links Notation, generalized
parametric intents, substitution-rule (`replace`/`when … do …`) handlers over
link CRUD, natural-language access to memory/APIs/code execution, a general
code-modifying/executing agent, and industry benchmark datasets — **now all
closed and merged** (PRs #305-#311). A fourth audit (2026-05-27) found the
remaining gap was the **generality of the synthesis step** and opened the
synthesis batch E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317)):
a general link-native synthesis substrate, computed math/word-problem and
counting answers, general program synthesis from spec+tests, general text
manipulation, and a grown/ratcheted benchmark suite — **now all closed and
merged** (PRs #319-#323), with the benchmark suite passing 10/10. A fifth audit
(2026-05-29, PR #245 feedback) found the remaining gap is **parity** and opened
the parity batch E33-E34 ([#326](https://github.com/link-assistant/formal-ai/issues/326)-[#327](https://github.com/link-assistant/formal-ai/issues/327)):
a single shared, data-driven multilingual operation vocabulary so every handler
triggers equally in `en|ru|hi|zh`, and a cross-runtime sweep so the JavaScript
browser worker derives the same answers as the Rust core — **now all closed and
merged** (PRs #328-#329). With E1-E34 all merged, no vision-planning epic remains
open for issue #244.

| ID | Requirement | Status |
| --- | --- | --- |
| R250 | Documentation must fully track the implementation progress of every vision pillar and stay in sync with the actual code. | Implemented by adding and then repeatedly refreshing `ROADMAP.md`: the 2026-05-29 fifth-pass audit records that E1-E32 closed (zero tracked `#[ignore]` specification tests remain, the synthesis step derives answers, and the benchmark suite passes 10/10 with a ratchet), and the 2026-05-29 sixth-pass audit records that the E33-E34 parity batch is **now also closed and merged** (PRs #328/#329), closing the remaining cross-language and cross-runtime gap so no vision-planning epic remains open. |
| R251 | Stale documentation references must be reconciled with the real state of the code. | Implemented by reconciling `ARCHITECTURE.md` §16/§17, `REQUIREMENTS.md`, `VISION.md`, and the replaced pre-implementation roadmap, and grounding the post-merge status in `docs/case-studies/issue-244/raw-data/*`. |
| R252 | Issue data must be collected under `docs/case-studies/issue-244` and supplemented with online research. | Implemented with `raw-data/` snapshots (issue/PR/comments/CI and `raw-data/issue-survey.md`) and `raw-data/online-research.md` (Abstract Wikipedia/Wikifunctions, OpenCog AtomSpace/Hyperon, Lean/Z3, neuro-symbolic KG surveys, SWE-bench/HumanEval/MBPP/GSM8K/MATH dataset licensing), summarized and cited per `NON-GOALS.md`. |
| R253 | A deep case study must list each and all requirements from the issue and propose a solution plan per requirement, checking existing components/libraries. | Implemented by `docs/case-studies/issue-244/README.md` and `proposed-issues.md`; both cover the E1-E14, E15-E20, E21-E27, E28-E32, and E33-E34 batches with per-requirement plans and existing-component checks. |
| R254 | Critical problems that block the vision must be planned to be fixed first, on a solid foundation. | Implemented by the completed foundation batches E1-E27, the E28-E32 synthesis sequence (ordered foundation-first: the general link-native synthesis substrate before the per-domain math, program, and text synthesis that build on it), and the E33-E34 parity batch, which builds on the now-general synthesis step rather than re-implementing it per runtime/language. |
| R255 | All the issues needed to fully implement the vision must be created and recorded. | Implemented by opening E1-E14 ([#246](https://github.com/link-assistant/formal-ai/issues/246)-[#259](https://github.com/link-assistant/formal-ai/issues/259)), E15-E20 ([#278](https://github.com/link-assistant/formal-ai/issues/278)-[#283](https://github.com/link-assistant/formal-ai/issues/283)), the E21-E27 reasoning batch ([#298](https://github.com/link-assistant/formal-ai/issues/298)-[#304](https://github.com/link-assistant/formal-ai/issues/304), closed by PRs #305-#311), the E28-E32 synthesis batch ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317), closed by PRs #319-#323), and the E33-E34 parity batch ([#326](https://github.com/link-assistant/formal-ai/issues/326)-[#327](https://github.com/link-assistant/formal-ai/issues/327), closed by PRs #328-#329), all recorded in `docs/case-studies/issue-244/proposed-issues.md` and opened on GitHub. |
