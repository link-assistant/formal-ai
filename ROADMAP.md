# Roadmap: Implementation Progress Toward The Vision

This file is the single source of truth for how much of `VISION.md` is actually
built. It was introduced for issue
[#244](https://github.com/link-assistant/formal-ai/issues/244) and refreshed on
2026-05-26 in three passes (after the first planning batch E1-E14 merged to
`main`, after the follow-up batch E15-E20 merged, and again when the
reasoning-focused batch E21-E27 was opened), on 2026-05-27 for a fourth pass
(the E21-E27 reasoning batch closed, exposing the synthesis-generality gap), and
on 2026-05-29 for a fifth pass: the synthesis batch **E28-E32** is now also
closed and merged, the synthesis step **derives** answers instead of seeding
them, and the imported industry-benchmark suite passes **10/10** with a ratchet
floor. The fifth pass recorded the next gap — **cross-runtime and cross-language
parity**. A sixth pass on 2026-05-29 records that the parity batch **E33-E34**
is now **also closed and merged** (PRs [#328](https://github.com/link-assistant/formal-ai/pull/328)
and [#329](https://github.com/link-assistant/formal-ai/pull/329)): the
text-manipulation handler triggers from a single shared, data-driven
multilingual operation vocabulary in every supported language, and the
JavaScript browser worker derives the same synthesis/numeric/program/text
answers as the Rust core, verified by a shared cross-runtime fixture. With
E1-E34 all merged, **no vision-planning epic remains open** for issue #244. A
seventh pass on 2026-05-31 records the separate issue #349 reverse-sort roadmap:
issues [#355](https://github.com/link-assistant/formal-ai/issues/355)-[#364](https://github.com/link-assistant/formal-ai/issues/364)
are closed and merged, and the epic [#365](https://github.com/link-assistant/formal-ai/issues/365)
is closed by the final verification report in PR
[#377](https://github.com/link-assistant/formal-ai/pull/377).

It complements the existing docs rather than replacing them:

- `VISION.md`, `GOALS.md`, and `NON-GOALS.md` describe what we are building and
  why.
- `REQUIREMENTS.md` is the per-issue requirement matrix.
- `ARCHITECTURE.md` describes how the implemented pipeline is wired.
- `ROADMAP.md` tracks current implementation status, closed planning issues,
  and the next batch of remaining requirements.

## 2026-05-26 Audit Snapshot

The original issue #244 plan created E1-E14 as issues
[#246](https://github.com/link-assistant/formal-ai/issues/246) through
[#259](https://github.com/link-assistant/formal-ai/issues/259). All fourteen are
now closed by merged PRs on `main`, and the follow-up issues
[#262](https://github.com/link-assistant/formal-ai/issues/262) and
[#272](https://github.com/link-assistant/formal-ai/issues/272) are also closed.

The post-merge audit found:

- 142 closed issues surveyed from GitHub on 2026-05-26.
- The E1-E14 planning issues are closed and backed by merged PRs #260, #261, and
  #263 through #275.
- `tests/unit/specification/` now has zero `#[ignore = "tracked requirement: ..."]`
  tests; the original 69 tracked tests were graduated into active coverage.
- The six scoped follow-ups opened as E15-E20 are **now also closed** by merged
  PRs: E15 [#278](https://github.com/link-assistant/formal-ai/issues/278) → #285,
  E16 [#279](https://github.com/link-assistant/formal-ai/issues/279) → #287,
  E17 [#280](https://github.com/link-assistant/formal-ai/issues/280) → #289,
  E18 [#281](https://github.com/link-assistant/formal-ai/issues/281) → #290,
  E19 [#282](https://github.com/link-assistant/formal-ai/issues/282) → #291,
  E20 [#283](https://github.com/link-assistant/formal-ai/issues/283) → #293.
- The 2026-05-26 third-pass audit (driven by issue #244 feedback) found that the
  solver still leaned on a **fixed intent catalogue** and tended to fall back to
  an "I can't answer that" opener on anything unmatched, instead of reasoning
  under unknowns. That reasoning batch **E21-E27**
  ([#298](https://github.com/link-assistant/formal-ai/issues/298)-[#304](https://github.com/link-assistant/formal-ai/issues/304))
  is **now closed** by merged PRs #305-#311 (see the Completed Planning Batch
  table). Every message is now formalized into a Links-Notation intent
  (`src/intent_formalization.rs`), unmatched prompts run a reasoning-under-unknowns
  loop (`src/solver_unknown_reasoning.rs`) instead of a canned opener, `write a
  program` is one parametric intent, substitution rules (`src/substitution.rs`)
  run over link CRUD, and a permissive industry-benchmark slice is imported
  (`data/benchmarks/industry-suite.lino`).
- The 2026-05-27 fourth-pass audit found the next gap was the **generality of the
  synthesis step itself**. The universal 11-step loop is the main path for every
  prompt (verified in `src/solver.rs::solve_with_history_probability_store_and_intent_cache`),
  but the candidate-synthesis step still resolved answers from seeded handlers
  rather than deriving them. That defined the **E28-E32** synthesis batch.
- The 2026-05-29 fifth-pass audit found the **E28-E32** synthesis batch
  ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317))
  is **now closed** by merged PRs #319-#323 (see the Completed Planning Batch
  table). The synthesis step now derives candidates by composing decomposed
  sub-results over the links network, arithmetic/word-problem and counting
  answers are computed, Python functions are synthesized from spec + tests and
  verified in the bounded agent workspace, and the benchmark suite grew from a
  5-case to a 10-case slice that passes **10/10** with a `minimum_pass_count`
  ratchet (verified by `cargo test issue_304_benchmark_suite_reports_pass_fail_counts`
  → `passed=10 failed=0 total=10 minimum_pass_count=10`). The fifth-pass audit
  recorded the next gap as **cross-runtime and cross-language parity**: at that
  time the JavaScript browser worker (`src/web/formal_ai_worker.js`) had not yet
  absorbed the E28-E31 reasoning capabilities present in the Rust core, and the
  program-synthesis and text-manipulation handlers triggered only on English
  keywords. That gap became the **E33-E34** parity batch, which the sixth pass
  records as **now closed and merged** (PRs #328-#329 — see the Parity Batch
  table below).

The raw audit data is preserved under
`docs/case-studies/issue-244/raw-data/`:

- `closed-issues-2026-05-26.json`
- `merged-prs-2026-05-26.json`
- `deferred-marker-search-2026-05-26.txt`
- `ignored-tracked-tests-2026-05-26.txt`
- `next-batch-issues-2026-05-26.txt`
- `code-audit.md` and `online-research.md` (third-pass reasoning audit and prior art)

## Vision Pillars

Status legend:

- **Built**: implemented and covered by active tests.
- **Partial**: useful implementation exists, but a named follow-up still owns a
  remaining part of the requirement.
- **Open**: not implemented beyond documentation or planning.

| # | Vision pillar | Current status | Evidence | Remaining work |
| --- | --- | --- | --- | --- |
| 1 | "The associative network is the AI": one doublet-links store is the source of truth | Built | `src/link_store.rs`, `src/links_format.rs`, active `links_network` specs | `doublets-rs` made the default native physical store by [#278](https://github.com/link-assistant/formal-ai/issues/278) (PR #285). |
| 2 | Universal problem-solving loop runs for every prompt in the same shape | Built | `src/solver.rs::UniversalSolver`, active `reasoning_loop` specs | None in the E1-E14 backlog. |
| 3 | Formalization to Wikidata P-ids/Q-ids with fallback sources | Built | `src/translation/formalization.rs`, `src/translation/pipeline.rs`, active `formalization` specs | Future ranking improvements feed into [#279](https://github.com/link-assistant/formal-ai/issues/279). |
| 4 | Temperature-based interpretation selection plus clarify-vs-guess | Built | `src/translation/selection.rs`, `SolverConfig::temperature`, active tests | None in the E1-E14 backlog. |
| 5 | Public knowledge as a cache with provenance | Built | `src/solver.rs` and `src/solver_handlers/mod.rs` source-cache handling, active `source_cache` specs; `src/knowledge.rs` adds the coding oracle that treats Rosetta Code / Wikifunctions / the Hello World Collection / Stack Overflow as cached external APIs under a `min(1%, 512)` per-source cap ([#412](https://github.com/link-assistant/formal-ai/issues/412)) | None in the E1-E14 backlog; the oracle's gated live-refresh path follows the existing `FORMAL_AI_LIVE_API` discipline. |
| 6 | Translation through link-native meanings | Built | `src/translation/`, active `translation_via_links` specs, issue #526 `translation_round_trip` matrix, and `docs/case-studies/issue-526/` | None in the E1-E14 backlog; issue #526 now pins round-trip survival as the quality guard. |
| 7 | Code generation and cross-language translation | Built | `src/solver_handlers/software_project.rs`, active `code_generation` specs, Rust <-> JavaScript code-meaning round-trip coverage in `translation_via_links`; `src/solver_handler_oracle.rs` generalises `write_program` to languages the verified catalogue does not template (Kotlin/Swift/PHP/Bash/Lua/Haskell) by sourcing reviewed snippets from the cached knowledge oracle ([#412](https://github.com/link-assistant/formal-ai/issues/412)) | A task-agnostic meta-builder ("algorithm that builds algorithms", R7) is the tracked next step in [`docs/case-studies/issue-412`](docs/case-studies/issue-412/README.md). |
| 8 | Formal reasoning beyond a fixed answer table | Built | `src/proof_engine/decision.rs`, boolean and linear decision modules | Optional future backends can build on this, but #253 closed the planned requirement. |
| 9 | Chat over experience: why, facts, export, retraction | Built | `src/event_log.rs`, active `transparent_state` specs | None in the E1-E14 backlog. |
| 10 | Links-network invariants and dynamic type system | Built | `src/link_store.rs`, `src/links_format.rs`, active `links_network` specs | Native physical-store default is tracked separately in [#278](https://github.com/link-assistant/formal-ai/issues/278). |
| 11 | Bounded chat autonomy plus explicit isolated agent mode | Built | `src/solver.rs`, agent isolation specs, API gating | None in the E1-E14 backlog. |
| 12 | OpenAI-compatible API with auth and tool-call gating | Built | `src/protocol.rs`, `src/server.rs`, active `openai_compatibility` specs | None in the E1-E14 backlog. |
| 13 | Visual network beside chat and trace links on every surface | Built | `src/web/app.js`, `/v1/graph`, Telegram trace specs | None in the E1-E14 backlog. |
| 14 | Five rule shapes ending in compiled natural-language skills | Built | `src/skill_compiler.rs` typed/multi-step compiler with native lowering | Trigger/response generalized by [#283](https://github.com/link-assistant/formal-ai/issues/283) (PR #293). General substitution rules tracked separately as E24 [#301](https://github.com/link-assistant/formal-ai/issues/301). |
| 15 | Symbolic probabilistic ranking over the links network | Built | `src/probability.rs`, temperature selection plus symbolic evidence | Implemented by [#279](https://github.com/link-assistant/formal-ai/issues/279) (PR #287). |
| 16 | Desktop application path | Built | `desktop/` Electron shell | Packaged by [#280](https://github.com/link-assistant/formal-ai/issues/280) (PR #289). |
| 17 | Reusable associative packages, handlers, permissions, triggers | Built | `src/associative_package.rs`, handler registry, tool-call gating | Unified by [#281](https://github.com/link-assistant/formal-ai/issues/281) (PR #290). |
| 18 | Rust-to-WebAssembly parity with JavaScript reserved for UI/glue | Built | `src/web_engine_core.rs` plus the browser worker | Worker logic moved into Rust/WASM by [#282](https://github.com/link-assistant/formal-ai/issues/282) (PR #291). |
| 19 | Reasoning under unknowns rather than a canned fallback | Built | `src/solver_unknown_reasoning.rs`, active `unknown_reasoning` specs record `reasoning:known:` / `reasoning:unknown:` / `reasoning:candidate_source:` / `reasoning:gather_attempt:` and ask at most one minimal question | Implemented by E21 [#298](https://github.com/link-assistant/formal-ai/issues/298) (PR #305). The synthesis it falls into is still seeded — see pillar 26. |
| 20 | Routing by formalized intent, not a fixed catalogue | Built | `src/intent_formalization.rs`, `src/solver_formalization.rs`, active `intent_formalization` specs; every prompt is formalized into a Links-Notation intent and prior reasoning is cached | Implemented by E22 [#299](https://github.com/link-assistant/formal-ai/issues/299) (PR #306). The specialized-handler precedence behind the formalized router is now data: E44 [#663](https://github.com/link-assistant/formal-ai/issues/663) retired the `SPECIALIZED_HANDLERS` constant into `data/seed/handler-precedence.lino`, joined with the Rust function pointers by `specialized_handlers()`. |
| 21 | Parametric intents instead of one intent per language | Built | `SelectedRule::WriteProgram` with `program_parameter:language` / `program_parameter:task`, active `code_generation` specs | Implemented by E23 [#300](https://github.com/link-assistant/formal-ai/issues/300) (PR #307). |
| 22 | Substitution-rule handlers over link CRUD | Built | `src/substitution.rs`, active `substitution_rules` specs (`replace x y`, `when n do m` over link CRUD) | Implemented by E24 [#301](https://github.com/link-assistant/formal-ai/issues/301) (PR #308). |
| 23 | Natural-language access to memory, APIs, and code execution | Built | `src/solver_handlers/`, active `natural_language_access` specs; permission-gated NL → query/call/execute paths | Implemented by E25 [#302](https://github.com/link-assistant/formal-ai/issues/302) (PR #309). |
| 24 | General code-modifying / executing agent (not a memorizer) | Built | `src/agent.rs` bounded/isolated workspace runs allowlisted commands; `src/solver_handlers/program_synthesis.rs` synthesizes a Python function from the spec, then verifies it by executing the assertions in the workspace | Workspace execution built by E26 [#303](https://github.com/link-assistant/formal-ai/issues/303) (PR #310); spec-driven synthesis + verification added by E30 [#315](https://github.com/link-assistant/formal-ai/issues/315) (PR #321). Triggering is still English-keyword gated — see the Next Planning Batch (language parity). |
| 25 | Measured against industry benchmark datasets | Built | `data/benchmarks/industry-suite.lino`, `tests/unit/specification/benchmarks.rs`; HumanEval/MBPP/GSM8K/MATH/BIG-bench slice runs deterministically in CI | Imported by E27 [#304](https://github.com/link-assistant/formal-ai/issues/304) (PR #311); grown to a 10-case slice and gated on a rising pass count by E32 [#317](https://github.com/link-assistant/formal-ai/issues/317) (PR #323). The suite now reports **10/10 passing** with a `minimum_pass_count` ratchet so progress cannot silently regress. |
| 26 | General synthesis: derive solutions for the benchmark domains instead of seeding them | Built | The benchmark suite passes 10/10; `record_candidates` composes decomposed sub-results over the links network (E28), arithmetic/word-problem and counting answers are computed (E29), and Python functions are synthesized and verified (E30) rather than keyed on the prompt | Made general by E28-E32 ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317)) (PRs #319-#323). Held-out paraphrased variants guard against per-case memorization. |

## Completed Planning Batch

| Epic | Issue | Closing PR | Result |
| --- | --- | --- | --- |
| E1 | [#246](https://github.com/link-assistant/formal-ai/issues/246) | [#260](https://github.com/link-assistant/formal-ai/pull/260) | Added the link-store boundary and doublet projection. |
| E2 | [#247](https://github.com/link-assistant/formal-ai/issues/247) | [#261](https://github.com/link-assistant/formal-ai/pull/261) | Made the universal reasoning loop the main solver path. |
| E3 | [#248](https://github.com/link-assistant/formal-ai/issues/248) | [#263](https://github.com/link-assistant/formal-ai/pull/263) | Added P/Q formalization support. |
| E4 | [#249](https://github.com/link-assistant/formal-ai/issues/249) | [#264](https://github.com/link-assistant/formal-ai/pull/264) | Added temperature selection and clarify-vs-guess behavior. |
| E5 | [#250](https://github.com/link-assistant/formal-ai/issues/250) | [#265](https://github.com/link-assistant/formal-ai/pull/265) | Added source cache records with provenance. |
| E6 | [#251](https://github.com/link-assistant/formal-ai/issues/251) | [#266](https://github.com/link-assistant/formal-ai/pull/266) | Added translation through link-native meanings. |
| E7 | [#252](https://github.com/link-assistant/formal-ai/issues/252) | [#267](https://github.com/link-assistant/formal-ai/pull/267) | Expanded code generation and cross-language translation tests. |
| E8 | [#253](https://github.com/link-assistant/formal-ai/issues/253) | [#268](https://github.com/link-assistant/formal-ai/pull/268) | Added formal decision procedures. |
| E9 | [#254](https://github.com/link-assistant/formal-ai/issues/254) | [#269](https://github.com/link-assistant/formal-ai/pull/269) | Added chat-over-experience queries. |
| E10 | [#255](https://github.com/link-assistant/formal-ai/issues/255) | [#270](https://github.com/link-assistant/formal-ai/pull/270) | Added links-network invariants and dynamic type tests. |
| E11 | [#256](https://github.com/link-assistant/formal-ai/issues/256) | [#271](https://github.com/link-assistant/formal-ai/pull/271) | Added isolated agent-mode controls. |
| E12 | [#257](https://github.com/link-assistant/formal-ai/issues/257) | [#273](https://github.com/link-assistant/formal-ai/pull/273) | Added API authentication and tool-call gating. |
| E13 | [#258](https://github.com/link-assistant/formal-ai/issues/258) | [#274](https://github.com/link-assistant/formal-ai/pull/274) | Added network visualization and trace surfaces. |
| E14 | [#259](https://github.com/link-assistant/formal-ai/issues/259) | [#275](https://github.com/link-assistant/formal-ai/pull/275) | Added deterministic natural-language trigger/response skill compilation. |
| E15 | [#278](https://github.com/link-assistant/formal-ai/issues/278) | [#285](https://github.com/link-assistant/formal-ai/pull/285) | Made `doublets-rs` the default native physical store. |
| E16 | [#279](https://github.com/link-assistant/formal-ai/issues/279) | [#287](https://github.com/link-assistant/formal-ai/pull/287) | Added symbolic probabilistic evidence over Links Notation. |
| E17 | [#280](https://github.com/link-assistant/formal-ai/issues/280) | [#289](https://github.com/link-assistant/formal-ai/pull/289) | Added the Electron desktop shell. |
| E18 | [#281](https://github.com/link-assistant/formal-ai/issues/281) | [#290](https://github.com/link-assistant/formal-ai/pull/290) | Unified reusable associative packages and the permission model. |
| E19 | [#282](https://github.com/link-assistant/formal-ai/issues/282) | [#291](https://github.com/link-assistant/formal-ai/pull/291) | Completed Rust-to-WebAssembly solver parity for the browser worker. |
| E20 | [#283](https://github.com/link-assistant/formal-ai/issues/283) | [#293](https://github.com/link-assistant/formal-ai/pull/293) | Generalized the skill compiler beyond trigger/response. |
| E21 | [#298](https://github.com/link-assistant/formal-ai/issues/298) | [#305](https://github.com/link-assistant/formal-ai/pull/305) | Reason under unknowns (state known/unknown, gather, ask one minimal question) instead of a canned fallback. |
| E22 | [#299](https://github.com/link-assistant/formal-ai/issues/299) | [#306](https://github.com/link-assistant/formal-ai/pull/306) | Formalize every message into a Links-Notation intent and cache prior reasoning. |
| E23 | [#300](https://github.com/link-assistant/formal-ai/issues/300) | [#307](https://github.com/link-assistant/formal-ai/pull/307) | Collapse per-language program intents into one parametric `write a program` intent. |
| E24 | [#301](https://github.com/link-assistant/formal-ai/issues/301) | [#308](https://github.com/link-assistant/formal-ai/pull/308) | Add `link-cli`-style `replace x y` / `when n do m` substitution rules over link CRUD. |
| E25 | [#302](https://github.com/link-assistant/formal-ai/issues/302) | [#309](https://github.com/link-assistant/formal-ai/pull/309) | Gate natural-language access to memory, APIs, and code execution. |
| E26 | [#303](https://github.com/link-assistant/formal-ai/issues/303) | [#310](https://github.com/link-assistant/formal-ai/pull/310) | Add the bounded, isolated agent workspace that runs allowlisted commands. |
| E27 | [#304](https://github.com/link-assistant/formal-ai/issues/304) | [#311](https://github.com/link-assistant/formal-ai/pull/311) | Import a permissive industry-benchmark slice (HumanEval/MBPP/GSM8K/MATH/BIG-bench). |
| E28 | [#313](https://github.com/link-assistant/formal-ai/issues/313) | [#319](https://github.com/link-assistant/formal-ai/pull/319) | Derive synthesis candidates by composing decomposed sub-results over the links network instead of returning seeded answers. |
| E29 | [#314](https://github.com/link-assistant/formal-ai/issues/314) | [#320](https://github.com/link-assistant/formal-ai/pull/320) | Compute math/word-problem and counting answers (GSM8K, MATH, BIG-bench) deterministically rather than seeding them. |
| E30 | [#315](https://github.com/link-assistant/formal-ai/issues/315) | [#321](https://github.com/link-assistant/formal-ai/pull/321) | Synthesize HumanEval/MBPP Python functions from spec + tests and verify them in the bounded agent workspace. |
| E31 | [#316](https://github.com/link-assistant/formal-ai/issues/316) | [#322](https://github.com/link-assistant/formal-ai/pull/322) | Generalize text manipulation over arbitrary user input (transform/extract/count/rewrite). |
| E32 | [#317](https://github.com/link-assistant/formal-ai/issues/317) | [#323](https://github.com/link-assistant/formal-ai/pull/323) | Grow the benchmark suite to 10 cases and gate progress on a rising pass-count ratchet with held-out variants. |

Issues [#262](https://github.com/link-assistant/formal-ai/issues/262) and
[#272](https://github.com/link-assistant/formal-ai/issues/272) were closed by
[#276](https://github.com/link-assistant/formal-ai/pull/276) and
[#277](https://github.com/link-assistant/formal-ai/pull/277). They are outside
the E1-E14 vision batch.

## Parity Batch (E33-E34) — closed and merged

The fifth-pass 2026-05-29 audit (issue #244 PR feedback) found that storage,
surfaces, routing, reasoning scaffolding, and synthesis generality are all built
and the benchmark suite passes 10/10. The remaining gap was **parity** — the
explicit ask that "all Rust and JavaScript logic are in sync" and "all languages
are supported equally". The sixth-pass audit (also 2026-05-29) records that both
sub-gaps are now **closed and merged**:

1. **Cross-language parity — closed by E33 ([#326](https://github.com/link-assistant/formal-ai/issues/326), PR [#328](https://github.com/link-assistant/formal-ai/pull/328)).**
   The text-manipulation handler no longer triggers on English literals. Every
   operation is recognised by canonicalising the prompt against one shared,
   data-driven table (`data/seed/operation-vocabulary.lino`) that lists each
   operation's surface forms per supported language (`en|ru|hi|zh`). Adding a
   surface form — or a whole new language — is a **seed-data edit, not a code
   change**. The Rust core loads it via `seed::operation_vocabulary()`; the
   browser worker loads the same file via `src/web/seed_loader.js`.
2. **Cross-runtime parity — closed by E34 ([#327](https://github.com/link-assistant/formal-ai/issues/327), PR [#329](https://github.com/link-assistant/formal-ai/pull/329)).**
   The JavaScript browser worker (`src/web/formal_ai_worker.js`) now routes
   synthesis prompts through `tryLinkNativeSynthesis`, `tryProgramSynthesis`, and
   `tryTextManipulation`, deriving the same synthesis/numeric/program/text
   answers as the Rust core. Parity is pinned by the shared fixture
   `data/parity/cross-runtime-synthesis.json`, the Rust test
   `shared_cross_runtime_synthesis_fixture_matches_rust_solver`, and the browser
   e2e `tests/e2e/tests/issue-327.spec.js`, all of which enforce the
   anti-memorization rule (forbidden literal answers must not appear).

With E1-E34 all closed and merged, **no vision-planning epic remains open** for
issue #244. The deep per-requirement plan lives in
`docs/case-studies/issue-244/proposed-issues.md`.

| Epic | Issue | Vision gap | Status |
| --- | --- | --- | --- |
| E33 | [#326](https://github.com/link-assistant/formal-ai/issues/326) | Universal multilingual operation vocabulary: every reasoning handler triggers equally in `en\|ru\|hi\|zh` via one shared data-driven vocabulary, not per-handler English literals | Closed / merged (PR [#328](https://github.com/link-assistant/formal-ai/pull/328)) |
| E34 | [#327](https://github.com/link-assistant/formal-ai/issues/327) | Cross-runtime parity: the JS browser worker derives synthesis/numeric/program/text answers exactly as the Rust core does (E28-E31), verified by shared parity tests | Closed / merged (PR [#329](https://github.com/link-assistant/formal-ai/pull/329)) |

## Issue #349 Reverse-Sort Roadmap - closed and merged

The issue #349 roadmap fixed the reported multi-turn Russian coding dialog:
after the assistant wrote a Rust file-listing program and modified it to accept
a path argument, the follow-up "Сделай сортировку результатов в обратном
порядке" must produce a reverse-sorted program modification instead of
`intent: unknown`.

Issue [#365](https://github.com/link-assistant/formal-ai/issues/365) is the
final tracker for this roadmap. Its blockers are all closed, and the closure
evidence is recorded in
[`docs/case-studies/issue-365/README.md`](docs/case-studies/issue-365/README.md).

| Issue | Closing PR | Result |
| --- | --- | --- |
| [#355](https://github.com/link-assistant/formal-ai/issues/355) | [#366](https://github.com/link-assistant/formal-ai/pull/366) | Active #349 reverse-sort integration reproduction. |
| [#356](https://github.com/link-assistant/formal-ai/issues/356) | [#367](https://github.com/link-assistant/formal-ai/pull/367) | Rule-synthesis design over Links Notation. |
| [#357](https://github.com/link-assistant/formal-ai/issues/357) | [#369](https://github.com/link-assistant/formal-ai/pull/369) | Active-program coreference for bare program-result follow-ups. |
| [#358](https://github.com/link-assistant/formal-ai/issues/358) | [#370](https://github.com/link-assistant/formal-ai/pull/370) | Composable program modifiers, including `reverse_sort`. |
| [#359](https://github.com/link-assistant/formal-ai/issues/359) | [#371](https://github.com/link-assistant/formal-ai/pull/371) | Rule construction for unknown program follow-ups. |
| [#360](https://github.com/link-assistant/formal-ai/issues/360) | [#372](https://github.com/link-assistant/formal-ai/pull/372) | Default-off diagnostics for the full write-program reasoning chain. |
| [#361](https://github.com/link-assistant/formal-ai/issues/361) | [#373](https://github.com/link-assistant/formal-ai/pull/373) | Rust/browser-worker parity for the #349 flow. |
| [#362](https://github.com/link-assistant/formal-ai/issues/362) | [#374](https://github.com/link-assistant/formal-ai/pull/374) | Multilingual coding-modification benchmark ratchet. |
| [#363](https://github.com/link-assistant/formal-ai/issues/363) | [#375](https://github.com/link-assistant/formal-ai/pull/375) | Reasoning-first report behavior. |
| [#364](https://github.com/link-assistant/formal-ai/issues/364) | [#376](https://github.com/link-assistant/formal-ai/pull/376) | White-box unknown-trace self-improvement loop. |
| [#365](https://github.com/link-assistant/formal-ai/issues/365) | [#377](https://github.com/link-assistant/formal-ai/pull/377) | Final closure report and verification map. |

## Issue #408 Text And Code Editing - merged (PR #416)

PR [#416](https://github.com/link-assistant/formal-ai/pull/416) fixes the issue
[#408](https://github.com/link-assistant/formal-ai/issues/408) Russian follow-up
replacement failure by routing text and code edit requests through a
deterministic symbolic edit path. The branch verifies the original reproduction,
multilingual replacement variants, punctuation-tolerant replacements, broader
case, extraction, counting, punctuation, and line-shape edit operations, 61
self-authored benchmark-family edit examples, and a manifest-backed 48-source
local profile with 30 deterministic variations per source.

The issue #408 local benchmark ratchet passes 1,440 of 1,440 generated profile
checks in `issue_408_text_code_edit_profile_passes_local_ratchet`. Each of the
48 researched sources has an explicit repository-local 10% floor of 3 checks and
the stronger ratchet requires 30/30 per source, so the benchmark work requested
for #408 is closed in this PR.

## Issue #538 Detailed Meanings and Words - merged (PR #601)

PR [#601](https://github.com/link-assistant/formal-ai/pull/601) answers the
concrete, verifiable core of issue
[#538](https://github.com/link-assistant/formal-ai/issues/538): the tomato
meaning now records, from the seed data, whether each surface
(`tomato`/`tomatoes`, `помидор`/`помидоры`, `томат`/`томаты`) is singular or
plural and what part of speech it is; every surface denotes its meaning
(bidirectional word ⇄ meaning); the previously missing plural `томаты` is added
so both Russian synonyms are symmetric; and the grammatical values are grounded
in Wikidata (`Q104083`/`Q110786`/`Q146786`) and lexicalised in en/ru/hi/zh. This
is captured by `REQUIREMENTS.md` rows R370–R377 and `tests/unit/issue_538.rs`.

The issue also states a large aspirational programme. Rather than half-build it,
this PR decomposes it into tracked follow-ups (`REQUIREMENTS.md` R378–R386,
detailed in `docs/case-studies/issue-538/solution-plan.md`), each with a smallest
next step:

- **Bulk semantics import** (R378) — generalize `scripts/ground-meanings.rs` into
  a batch importer that emits the enriched-surface template the tomato block now
  shows by hand.
- **Hardcoded-string audit** (R379) — a CI lint over `src/` that burns down an
  allowlist of user-facing literals not routed through the lexicon, building on
  `docs/design/no-hardcoded-natural-language.md`.
- **Rust→WASM worker** (R380) — widen the existing demo WASM worker
  (`src/web/wasm-worker/`, issue #1 R16) to absorb the remaining
  `src/web/worker/*.js` logic; the build target already exists.
- **CST/AST in data** (R381) — round-trip one module's `syn` AST into `.lino`
  before scaling to the whole crate.
- **Mermaid diagrams** (R382) — generate one diagram from the existing method
  registry as a build artifact.
- **Interactive debug view** (R383) — extend the exploratory notes under
  `docs/vscode/`.
- **Self-inspecting universal meta algorithm & contradiction warnings** (R384) —
  overlaps issue #559; this issue's own mixed scope is a ready contradiction
  fixture.
- **Agent-CLI self-hosting** (R385) — script a single Agent-CLI session that
  reproduces one atomic edit (e.g. adding `томаты`) in a scratch repo and capture
  its session JSON; documented as the way-forward in `CONTRIBUTING.md`.

## Issue #526 Translation Quality - merged (PR #635)

PR [#635](https://github.com/link-assistant/formal-ai/pull/635) closes the
translation-quality gap from issue
[#526](https://github.com/link-assistant/formal-ai/issues/526): round-trip
survival is now the regression contract. Natural-language tests cover
language-to-meta-to-same-language survival and every directed pair across en,
ru, hi, and zh using the seeded apple meaning. Code translation now normalizes
the simple add-function slice to one code meaning and verifies Rust ->
JavaScript -> Rust preserves the same `meaning:` evidence link. The case study
and online research live in `docs/case-studies/issue-526/`.

## 2026-07-14 Requirement-Status Audit (issue #651)

An eighth pass on 2026-07-14 audited **all 329 closed issues and all 317 merged
PRs** against the maintainer's original requirements. Per issue
[#651](https://github.com/link-assistant/formal-ai/issues/651), this roadmap now
tracks **requirements, not issues**: each area below carries an honest
done / partial / not-done status. The audit's headline finding is that the
dominant historical failure mode was *silent scope-narrowing* — the reported
prompt got fixed while the attached generalization, benchmark, or integration
requirement was dropped. The consolidated regression backlog is tracked in
[#710](https://github.com/link-assistant/formal-ai/issues/710).

Requirement-level status by area:

| Area (standing requirement) | Status | Owning issues |
| --- | --- | --- |
| Universal 11-step solver runs for every prompt | Done | — (pillar 2) |
| Only memory + meta algorithm; no specialized Rust handlers (#559 mandate) | Not done (36 handler files, 82 `try_*` dispatch entries remain) | [#663](https://github.com/link-assistant/formal-ai/issues/663), [#699](https://github.com/link-assistant/formal-ai/issues/699) |
| Real upstream benchmarks with honest scores (10% → all; #408/#440/#303) | Not done (repository-local proxies only) | [#698](https://github.com/link-assistant/formal-ai/issues/698) |
| Self-improvement that compounds (promotion, adoption, self-hosting metric) | Partial (promotion and honest release metric exist; adoption remains unproven) | [#656](https://github.com/link-assistant/formal-ai/issues/656), [#657](https://github.com/link-assistant/formal-ai/issues/657), [#701](https://github.com/link-assistant/formal-ai/issues/701), [#705](https://github.com/link-assistant/formal-ai/issues/705) |
| Symbolic world models: current/target contexts, diff, sync, consequence prediction (#649) | Partial (design + substrate audit merged in PR #675; behaviors unimplemented) | [#686](https://github.com/link-assistant/formal-ai/issues/686), [#702](https://github.com/link-assistant/formal-ai/issues/702) |
| Agentic-CLI server correctness (tools fire by intent in every phrasing) | Partial (capability router merged for #680; write/read routing and qwen wire fix in flight) | [#681](https://github.com/link-assistant/formal-ai/issues/681), [#682](https://github.com/link-assistant/formal-ai/issues/682), [#671](https://github.com/link-assistant/formal-ai/issues/671), [#687](https://github.com/link-assistant/formal-ai/issues/687) |
| Formal AI as orchestrator of external agent CLIs (agent/claude/codex/gemini/qwen), Hive-Mind dispatch | Not done (#655's headline scenario blocked upstream) | [#703](https://github.com/link-assistant/formal-ai/issues/703) |
| Parallel candidate portfolios + budget-driven search (F4) | Not done | [#662](https://github.com/link-assistant/formal-ai/issues/662), [#704](https://github.com/link-assistant/formal-ai/issues/704) |
| Anticipatory learning (predict next requests, pre-learn) | Not done | [#705](https://github.com/link-assistant/formal-ai/issues/705) |
| "All languages" through the meta language (beyond en/ru/hi/zh) | Partial (4 seed languages; round-trip contract exists) | [#660](https://github.com/link-assistant/formal-ai/issues/660), [#706](https://github.com/link-assistant/formal-ai/issues/706) |
| General computer-use without vision (files/shell/structured web plans) | Partial (bounded agent + pinned recipes only) | [#707](https://github.com/link-assistant/formal-ai/issues/707) |
| Turing-complete NL memory queries (#529) | Partial (append + single substitution shipped) | [#708](https://github.com/link-assistant/formal-ai/issues/708) |
| Multi-source search fusion through the meta language (#505/#444/#63/#153) | Not done (routing only) | [#709](https://github.com/link-assistant/formal-ai/issues/709) |
| Measuring units via si-units (#439) | Not done | [#700](https://github.com/link-assistant/formal-ai/issues/700) |
| Silently-dropped chat/UX/process requirements re-verified | Not done (audit checklist compiled) | [#710](https://github.com/link-assistant/formal-ai/issues/710) |
| Data-is-the-interface hygiene (no hardcoded NL, links-network terminology, precedence in seed) | Partial | [#659](https://github.com/link-assistant/formal-ai/issues/659), [#663](https://github.com/link-assistant/formal-ai/issues/663), [#664](https://github.com/link-assistant/formal-ai/issues/664) |
| Delivery breadth (PWA, npm engine, VS Code Marketplace, debugger, WebVM, packages, cloud sync) | Not done | [#665](https://github.com/link-assistant/formal-ai/issues/665)-[#670](https://github.com/link-assistant/formal-ai/issues/670), [#658](https://github.com/link-assistant/formal-ai/issues/658) |
| Self-coding chain (workspace census → gated promotion → self-hosting metric) | Partial: #656 executes trusted gates and Agent-authored local-branch promotion; #657 now records the release metric; workspace census remains | [#673](https://github.com/link-assistant/formal-ai/issues/673) → [#656](https://github.com/link-assistant/formal-ai/issues/656) → [#657](https://github.com/link-assistant/formal-ai/issues/657) |

**Open planning batches.** The "no vision-planning epic remains open" statement
earlier in this file is scoped to issue #244 only. Two batches are open now:

- **E37–E55** ([#656](https://github.com/link-assistant/formal-ai/issues/656)–[#674](https://github.com/link-assistant/formal-ai/issues/674)),
  created from issue #651's gap analysis; first implementation PRs are in
  flight (#688–#697).
- **E56–E68** ([#698](https://github.com/link-assistant/formal-ai/issues/698)–[#710](https://github.com/link-assistant/formal-ai/issues/710)),
  created from the 2026-07-14 full-history requirement audit; all are
  sub-issues of #651 with explicit blocked-by relationships recorded through
  the GitHub dependencies API.

## Verification Contract

When any roadmap item changes, the PR should update the corresponding rows in
`REQUIREMENTS.md`, the architecture status table, and this file. If the change
closes a follow-up issue, remove or narrow the corresponding "remaining work"
entry instead of leaving stale future-work wording behind.
