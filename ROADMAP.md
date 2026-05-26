# Roadmap: Implementation Progress Toward The Vision

This file is the single source of truth for how much of `VISION.md` is actually
built. It was introduced for issue
[#244](https://github.com/link-assistant/formal-ai/issues/244) and refreshed on
2026-05-26 in three passes: after the first planning batch (E1-E14) merged to
`main`, after the follow-up batch (E15-E20) merged, and again when the
reasoning-focused batch (E21-E27) was opened.

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
- The remaining gap is no longer storage, surfaces, or compilation. The
  2026-05-26 third-pass audit (driven by issue #244 feedback) found that the
  solver still leans on a **fixed intent catalogue** and tends to fall back to an
  "I can't answer that" opener on anything unmatched, instead of reasoning under
  unknowns. That reasoning gap is owned by the new batch **E21-E27**
  ([#298](https://github.com/link-assistant/formal-ai/issues/298)-[#304](https://github.com/link-assistant/formal-ai/issues/304)).

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
| 5 | Public knowledge as a cache with provenance | Built | `src/solver.rs` and `src/solver_handlers/mod.rs` source-cache handling, active `source_cache` specs | None in the E1-E14 backlog. |
| 6 | Translation through link-native meanings | Built | `src/translation/`, active `translation_via_links` specs | None in the E1-E14 backlog. |
| 7 | Code generation and cross-language translation | Built | `src/solver_handlers/software_project.rs`, active `code_generation` specs | None in the E1-E14 backlog. |
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
| 19 | Reasoning under unknowns rather than a canned fallback | Open | `src/unknown_opener.rs` only varies a canned opener; no data-gathering or reasoning steps | Implement reasoning-under-unknowns in E21 [#298](https://github.com/link-assistant/formal-ai/issues/298). |
| 20 | Routing by formalized intent, not a fixed catalogue | Open | `src/engine.rs::select_rule_for` / `SelectedRule`, `src/solver.rs::SPECIALIZED_HANDLERS` are hardcoded; `FormalizationCandidate` is not the router | Formalize every message into a Links-Notation intent and cache prior reasoning in E22 [#299](https://github.com/link-assistant/formal-ai/issues/299). |
| 21 | Parametric intents instead of one intent per language | Open | `src/engine_hello_world.rs::HELLO_WORLD_PROGRAMS` enumerates ~10 languages | Collapse into a `write a program(language, task)` intent in E23 [#300](https://github.com/link-assistant/formal-ai/issues/300). |
| 22 | Substitution-rule handlers over link CRUD | Open | Handlers are Rust functions; skill compiler only does trigger/response | Add `link-cli`-style `replace x y` / `when n do m` rules as data on link CRUD in E24 [#301](https://github.com/link-assistant/formal-ai/issues/301). |
| 23 | Natural-language access to memory, APIs, and code execution | Open | A few specialized handlers exist; no general NL → query/call/execute path; no runtime code execution in core | Implement permissioned NL access in E25 [#302](https://github.com/link-assistant/formal-ai/issues/302). |
| 24 | General code-modifying / executing agent (not a memorizer) | Open | Agent mode is gated but never executes; programs are memorized seeds | Build the general coding agent with expanded tests in E26 [#303](https://github.com/link-assistant/formal-ai/issues/303). |
| 25 | Measured against industry benchmark datasets | Open | Only own seeds and specification tests; no external benchmarks | Import permissive programming/math/problem-solving benchmarks in E27 [#304](https://github.com/link-assistant/formal-ai/issues/304). |

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

Issues [#262](https://github.com/link-assistant/formal-ai/issues/262) and
[#272](https://github.com/link-assistant/formal-ai/issues/272) were closed by
[#276](https://github.com/link-assistant/formal-ai/pull/276) and
[#277](https://github.com/link-assistant/formal-ai/pull/277). They are outside
the E1-E14 vision batch.

## Next Planning Batch

The third-pass 2026-05-26 audit (issue #244 feedback) found the remaining gap is
**reasoning behaviour**, not storage or surfaces: the solver still routes on a
fixed intent catalogue and falls back to a canned opener on anything unmatched.
The E21-E27 batch moves the assistant from intent-matching toward reasoning. It
is ordered foundation-first — intent formalization and reasoning-under-unknowns
come before the general coding agent that depends on them. Each issue lists its
code-grounded acceptance criteria; the deep per-requirement plan lives in
`docs/case-studies/issue-244/proposed-issues.md`.

| Epic | Issue | Vision gap | Code anchor it must replace/extend |
| --- | --- | --- | --- |
| E21 | [#298](https://github.com/link-assistant/formal-ai/issues/298) | Reason under unknowns; gather missing data instead of failing | `src/unknown_opener.rs` canned fallback |
| E22 | [#299](https://github.com/link-assistant/formal-ai/issues/299) | Formalize every message into a Links-Notation intent; cache prior reasoning | `src/engine.rs::SelectedRule` / `src/solver.rs::SPECIALIZED_HANDLERS` |
| E23 | [#300](https://github.com/link-assistant/formal-ai/issues/300) | One parametric `write a program` intent, not one per language | `src/engine_hello_world.rs::HELLO_WORLD_PROGRAMS` |
| E24 | [#301](https://github.com/link-assistant/formal-ai/issues/301) | `link-cli`-style `replace x y` / `when n do m` rules over link CRUD | `src/skill_compiler.rs` trigger/response only |
| E25 | [#302](https://github.com/link-assistant/formal-ai/issues/302) | NL to query memory, call APIs, execute code (permissioned) | `SPECIALIZED_HANDLERS` http/web/exec stubs |
| E26 | [#303](https://github.com/link-assistant/formal-ai/issues/303) | General code-writing/modifying/executing agent + many more tests | gated-but-unexecuted agent mode |
| E27 | [#304](https://github.com/link-assistant/formal-ai/issues/304) | Import permissive programming/math/problem-solving benchmarks | own seeds + specification tests only |

## Verification Contract

When any roadmap item changes, the PR should update the corresponding rows in
`REQUIREMENTS.md`, the architecture status table, and this file. If the change
closes a follow-up issue, remove or narrow the corresponding "remaining work"
entry instead of leaving stale future-work wording behind.
