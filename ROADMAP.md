# Roadmap: Implementation Progress Toward The Vision

This file is the single source of truth for how much of `VISION.md` is actually
built. It was introduced for issue
[#244](https://github.com/link-assistant/formal-ai/issues/244) and updated again
on 2026-05-26 after the first planning batch was merged to `main`.

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
- Remaining gaps are no longer the original E1-E14 backlog. They are the six
  scoped follow-ups opened as E15-E20:
  [#278](https://github.com/link-assistant/formal-ai/issues/278),
  [#279](https://github.com/link-assistant/formal-ai/issues/279),
  [#280](https://github.com/link-assistant/formal-ai/issues/280),
  [#281](https://github.com/link-assistant/formal-ai/issues/281),
  [#282](https://github.com/link-assistant/formal-ai/issues/282), and
  [#283](https://github.com/link-assistant/formal-ai/issues/283).

The raw audit data is preserved under
`docs/case-studies/issue-244/raw-data/`:

- `closed-issues-2026-05-26.json`
- `merged-prs-2026-05-26.json`
- `deferred-marker-search-2026-05-26.txt`
- `ignored-tracked-tests-2026-05-26.txt`
- `next-batch-issues-2026-05-26.txt`

## Vision Pillars

Status legend:

- **Built**: implemented and covered by active tests.
- **Partial**: useful implementation exists, but a named follow-up still owns a
  remaining part of the requirement.
- **Open**: not implemented beyond documentation or planning.

| # | Vision pillar | Current status | Evidence | Remaining work |
| --- | --- | --- | --- | --- |
| 1 | "The associative network is the AI": one doublet-links store is the source of truth | Partial | `src/link_store.rs`, `src/links_format.rs`, active `links_network` specs | Make `doublets-rs` the default native physical store in [#278](https://github.com/link-assistant/formal-ai/issues/278). |
| 2 | Universal problem-solving loop runs for every prompt in the same shape | Built | `src/solver.rs::UniversalSolver`, active `reasoning_loop` specs | None in the E1-E14 backlog. |
| 3 | Formalization to Wikidata P-ids/Q-ids with fallback sources | Built | `src/translation/formalization.rs`, `src/translation/pipeline.rs`, active `formalization` specs | Future ranking improvements feed into [#279](https://github.com/link-assistant/formal-ai/issues/279). |
| 4 | Temperature-based interpretation selection plus clarify-vs-guess | Built | `src/translation/selection.rs`, `SolverConfig::temperature`, active tests | None in the E1-E14 backlog. |
| 5 | Public knowledge as a cache with provenance | Built | `src/source_cache.rs`, active `source_cache` specs | None in the E1-E14 backlog. |
| 6 | Translation through link-native meanings | Built | `src/translation/`, active `translation_via_links` specs | None in the E1-E14 backlog. |
| 7 | Code generation and cross-language translation | Built | `src/solver_handlers/software_project.rs`, active `code_generation` specs | None in the E1-E14 backlog. |
| 8 | Formal reasoning beyond a fixed answer table | Built | `src/proof_engine/decision.rs`, boolean and linear decision modules | Optional future backends can build on this, but #253 closed the planned requirement. |
| 9 | Chat over experience: why, facts, export, retraction | Built | `src/event_log.rs`, active `transparent_state` specs | None in the E1-E14 backlog. |
| 10 | Links-network invariants and dynamic type system | Built | `src/link_store.rs`, `src/links_format.rs`, active `links_network` specs | Native physical-store default is tracked separately in [#278](https://github.com/link-assistant/formal-ai/issues/278). |
| 11 | Bounded chat autonomy plus explicit isolated agent mode | Built | `src/solver.rs`, agent isolation specs, API gating | None in the E1-E14 backlog. |
| 12 | OpenAI-compatible API with auth and tool-call gating | Built | `src/protocol.rs`, `src/server.rs`, active `openai_compatibility` specs | None in the E1-E14 backlog. |
| 13 | Visual network beside chat and trace links on every surface | Built | `src/web/app.js`, `/v1/graph`, Telegram trace specs | None in the E1-E14 backlog. |
| 14 | Five rule shapes ending in compiled natural-language skills | Partial | `src/skill_compiler.rs` supports deterministic trigger/response packages | Generalized typed/multi-step compiler and native lowering in [#283](https://github.com/link-assistant/formal-ai/issues/283). |
| 15 | Symbolic probabilistic ranking over the links network | Open | Temperature selection exists, but Bayesian/Markov-style evidence is not implemented | Implement in [#279](https://github.com/link-assistant/formal-ai/issues/279). |
| 16 | Desktop application path | Open | CLI, HTTP, library, Telegram, and browser surfaces exist | Package the desktop wrapper in [#280](https://github.com/link-assistant/formal-ai/issues/280). |
| 17 | Reusable associative packages, handlers, permissions, triggers | Partial | Compiled skills, handler registry, and tool-call gating exist separately | Unify as package/permission records in [#281](https://github.com/link-assistant/formal-ai/issues/281). |
| 18 | Rust-to-WebAssembly parity with JavaScript reserved for UI/glue | Partial | `src/web_engine_core.rs` owns several browser-domain operations | Move remaining reusable worker logic into Rust/WASM in [#282](https://github.com/link-assistant/formal-ai/issues/282). |

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

Issues [#262](https://github.com/link-assistant/formal-ai/issues/262) and
[#272](https://github.com/link-assistant/formal-ai/issues/272) were closed by
[#276](https://github.com/link-assistant/formal-ai/pull/276) and
[#277](https://github.com/link-assistant/formal-ai/pull/277). They are outside
the E1-E14 vision batch.

## Next Planning Batch

The remaining requirements are deliberately smaller than the original epics.
They came from the 2026-05-26 audit of closed issues, stale deferred markers, and
architecture open questions.

| Epic | Issue | Requirement source | Why it remains |
| --- | --- | --- | --- |
| E15 | [#278](https://github.com/link-assistant/formal-ai/issues/278) | R60, `ARCHITECTURE.md` section 16 | `doublets-rs` exists behind the boundary, but is not yet the default native physical store. |
| E16 | [#279](https://github.com/link-assistant/formal-ai/issues/279) | R6 | Bayesian/Markov-style symbolic probabilistic evidence is not implemented. |
| E17 | [#280](https://github.com/link-assistant/formal-ai/issues/280) | R17 | No packaged desktop wrapper exists yet. |
| E18 | [#281](https://github.com/link-assistant/formal-ai/issues/281) | R65 | Package metadata, handler permissions, dependency records, and install/export/replay are not unified. |
| E19 | [#282](https://github.com/link-assistant/formal-ai/issues/282) | R194 | Browser-worker domain logic is only partially moved into Rust/WASM. |
| E20 | [#283](https://github.com/link-assistant/formal-ai/issues/283) | `ARCHITECTURE.md` section 16, R65 | Skill compilation supports trigger/response packages, not typed multi-step/native lowering. |

## Verification Contract

When any roadmap item changes, the PR should update the corresponding rows in
`REQUIREMENTS.md`, the architecture status table, and this file. If the change
closes a follow-up issue, remove or narrow the corresponding "remaining work"
entry instead of leaving stale future-work wording behind.
