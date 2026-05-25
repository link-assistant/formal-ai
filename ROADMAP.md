# Roadmap — Implementation Progress Toward The Vision

This file is the **single source of truth for how much of `VISION.md` is actually
built**. It exists because issue
[#244](https://github.com/link-assistant/formal-ai/issues/244) asks us to "update
our documentation to fully track the progress of implementation of all the
requirements" and keep "everything in docs … in sync with the actual state of the
code" before planning the remaining work.

It complements the existing docs rather than replacing them:

- `VISION.md` / `GOALS.md` / `NON-GOALS.md` — *what* we are building and *why*.
- `REQUIREMENTS.md` — the per-issue requirement matrix (R1 … R236).
- `ARCHITECTURE.md` — *how* the implemented pipeline is wired.
- **`ROADMAP.md` (this file)** — *how far along* each vision pillar is, mapped to
  the real code, the machine-checkable backlog, and the planning epic that closes
  the gap.

Status legend:

- ✅ **Built** — implemented and covered by green tests.
- 🟡 **Partial** — skeleton/real code exists but the vision invariant is not yet
  enforced; tracked by `#[ignore]` tests and/or an `ARCHITECTURE.md` §16 open
  question.
- ⛔ **Planned** — not implemented; only the contract (an `#[ignore]` test or an
  open question) exists.

The grounding for every status below is `docs/case-studies/issue-244/raw-data/code-audit.md`.
The planning epics (E1–E14) are specified in
`docs/case-studies/issue-244/proposed-issues.md`.

## 1. Vision pillars → real status → plan

| # | Vision pillar (`VISION.md`) | Status | Where it lives in `src/` | Gap (tracked tests / open question) | Epic |
| --- | --- | --- | --- | --- | --- |
| 1 | "The associative network is the AI": one doublet-links store is the source of truth | 🟡 | `memory.rs`, `memory/bundle.rs` (custom `.lino` `MemoryStore`); `event_log.rs` (in-process) | Durable store is **not** doublets yet; `links_network` storage invariants; ARCH §16.3 | **E1** |
| 2 | Universal problem-solving loop runs for *every* prompt in the same shape | 🟡 | `solver.rs` (11-step skeleton) but routing via 35+ keyword handlers in `solver_handlers/` | `reasoning_loop` (11); `chat_surface` impulse/trace/extend | **E2** |
| 3 | Formalization to Wikidata P-ids (verbs) / Q-ids (nouns), Wiktionary/Wikipedia fallback | 🟡 | `concepts.rs` (alias-based); `translation/wikidata.rs` (SPARQL) | Full P/Q extraction over arbitrary prompts; ARCH §16.1 | **E3** |
| 4 | Temperature-based interpretation selection + clarify-vs-guess (ask the fewest questions) | 🟡 | `SolverConfig.temperature`/`guess_probability`/`questioning_rigor` (knobs only) | No softmax/ε helper; ARCH §16.2 | **E4** |
| 5 | Public knowledge (Wikidata/Wikipedia/Wiktionary) as a **cache** with provenance | 🟡 | `translation/cache.rs`, `web_search_core.rs` | `source_cache` (8): url, `fetched_at`, TTL, hash, conflict, flush, offline | **E5** |
| 6 | Translation through link-native meanings (`formalize → meaning → deformalize`) | 🟡 | `translation/pipeline.rs`, `wiktionary.rs`, `wikidata.rs` (real pipeline) | `translation_via_links` (7): shared meaning id, intermediate meaning, untranslatable flag | **E6** |
| 7 | Code generation + cross-language translation in popular languages, TDD, honest execution | 🟡 | `solver_helpers.rs`, `solver_handlers/software_project*.rs`, hello-world seeds | `code_generation` (6): top-10, exec links, isolation, algorithm+tests, failure trace | **E7** |
| 8 | Formal reasoning that covers all test cases **and much more** (not a fixed table) | 🟡 | `proof_engine/` (classical-theorem registry) | General decision procedure (relative-meta-logic / SMT); issue Q9 | **E8** |
| 9 | Chat over experience: "why?", "what do you know about X?", "list my facts", export, retraction | 🟡 | `event_log.rs`, network-query handler | `transparent_state` (8) | **E9** |
| 10 | Links-network invariants + dynamic type system (`Type→SubType→Value`) | 🟡 | `seed.rs` (`.lino` type seeds), `event_log.rs` traces | `links_network` remainder (4): subtype chains, every fact a source link, every answer a trace, ordered steps | **E10** |
| 11 | Bounded chat autonomy + explicit agent mode with isolated execution | ⛔ | `SolverConfig.agent_mode` (guarded, never executes); `telegram_runtime.rs` (DinD) | `agent_isolation` (9); `chat_surface` refuse-unbounded | **E11** |
| 12 | OpenAI-compatible API with auth + tool-call gating | 🟡 | `protocol.rs`, HTTP server in `main.rs` | `openai_compatibility` (2): bearer auth, refuse tool call without agent mode | **E12** |
| 13 | Visual network beside chat + trace links on every surface | 🟡 | `web/`, `telegram.rs` | `network_visualization` (1); `telegram_surface` (1); `chat_surface` exec-status + diagnostics-off (2) | **E13** |
| 14 | Five rule shapes ending in compiled natural-language skills | 🟡 | `seed/` rule seeds, `solver_handlers/` (compiled form) | No skill compiler; ARCH §16.4 | **E14** |

## 2. Already built (the regression floor — Q12)

These behaviors are implemented and covered by **green** tests; no epic may
remove them. They are the floor the plan builds on.

- ✅ OpenAI-shaped Chat Completions + Responses over CLI and HTTP
  (`protocol.rs`, `main.rs`; `REQUIREMENTS.md` R2–R4).
- ✅ Deterministic symbolic answers with no neural inference (`engine.rs`; R1, R5).
- ✅ Shared `data/seed/*.lino` read identically by library, CLI, server, WASM,
  Telegram (`seed.rs`, `web/seed_loader.js`; R97–R104).
- ✅ Real translation pipeline with surface-formatting preservation
  (`translation/`; Issue #207, `multilingual.rs` green).
- ✅ Calculator delegation (`calculation.rs`, `link-calculator`;
  `calculator_delegation.rs` green; Issue #96).
- ✅ Classical-theorem proof presentation (`proof_engine/`).
- ✅ Full-memory bundle export/import across surfaces (`memory/bundle.rs`;
  Issue #18, #196).
- ✅ Telegram surface incl. Docker-in-Docker runtime (`telegram*.rs`; Issue #195).
- ✅ GitHub Pages WASM demo (`web/`; R15–R16).
- ✅ Self-aware environment directory + library-first API
  (`seed.rs::environment_directory`; VISION "Self-Aware Environments",
  "Library-First Availability").
- ✅ Green specification suites that encode current behavior:
  `capabilities.rs`, `multilingual.rs`, `prompt_variations.rs`,
  `reasoning_paths.rs`, `definition_fusion.rs`, `issue_146.rs`,
  `calculator_delegation.rs`, `project_lookups.rs`, `summarization_pipeline.rs`.

## 3. Machine-checkable backlog (the 69 tracked tests)

The vision gaps are encoded as **69 `#[ignore]` "tracked requirement" tests** in
`tests/unit/specification/`. Graduating a test out of `#[ignore]` is the
definition of done for the corresponding pillar.

| Spec file | Ignored tests | Pillar | Epic |
| --- | --- | --- | --- |
| `reasoning_loop.rs` | 11 | Universal loop | E2 |
| `links_network.rs` | 10 | Network store + type system | E1 (6) + E10 (4) |
| `agent_isolation.rs` | 9 | Agent mode | E11 |
| `transparent_state.rs` | 8 | Chat over experience | E9 |
| `source_cache.rs` | 8 | Knowledge cache | E5 |
| `translation_via_links.rs` | 7 | Meaning-anchored translation | E6 |
| `code_generation.rs` | 6 | Code generation | E7 |
| `chat_surface.rs` | 6 | Loop + agent + surfaces | E2 (3) + E11 (1) + E13 (2) |
| `openai_compatibility.rs` | 2 | API auth/gating | E12 |
| `telegram_surface.rs` | 1 | Trace on Telegram | E13 |
| `network_visualization.rs` | 1 | Graph beside chat | E13 |
| **Total** | **69** | | |

`ARCHITECTURE.md` §16 open questions map to epics too: §16.1 → E3,
§16.2 → E4, §16.3 → E1, §16.4 → E14.

## 4. Sequencing

Foundation epics first (issue Q13 — "fix critical problems first … solid
foundation"):

1. **E1** — unified doublet store (blocks E2, E5, E6, E9, E10, E13).
2. **E2** — universal loop as the only entry path (blocks E3, E4, E6, E7, E8, E9).
3. **E3** — formalization engine (blocks E6, E10).
4. Then, in parallel where dependencies allow: **E4, E5, E10**, followed by
   **E6, E7, E8, E9, E14**, and the surface/agent epics **E11 → E12 → E13**.

```
E1 ──► E2 ──► E3 ──► E6, E10
 │      │      └────► E4
 ├────► E5
 ├────► E9
 └────► E13
        E2 ──► E7, E8, E14
        E2 ──► E11 ──► E12 ──► E13
```

## 5. Tracking issues

| Epic | Title | Issue |
| --- | --- | --- |
| E1 | Unified doublet-links store (doublets-rs + doublets-web) | [#246](https://github.com/link-assistant/formal-ai/issues/246) |
| E2 | Make the universal reasoning loop the only entry path | [#247](https://github.com/link-assistant/formal-ai/issues/247) |
| E3 | Full Wikidata P/Q-id formalization engine | [#248](https://github.com/link-assistant/formal-ai/issues/248) |
| E4 | Temperature-based interpretation selection + clarify-vs-guess | [#249](https://github.com/link-assistant/formal-ai/issues/249) |
| E5 | Public-knowledge source cache with provenance | [#250](https://github.com/link-assistant/formal-ai/issues/250) |
| E6 | Translation via link-native meanings | [#251](https://github.com/link-assistant/formal-ai/issues/251) |
| E7 | Code generation & cross-language translation | [#252](https://github.com/link-assistant/formal-ai/issues/252) |
| E8 | Formal reasoning engine (relative-meta-logic / SMT) | [#253](https://github.com/link-assistant/formal-ai/issues/253) |
| E9 | Chat-over-experience queries | [#254](https://github.com/link-assistant/formal-ai/issues/254) |
| E10 | Links-network invariants & dynamic type system | [#255](https://github.com/link-assistant/formal-ai/issues/255) |
| E11 | Agent mode with isolated execution | [#256](https://github.com/link-assistant/formal-ai/issues/256) |
| E12 | Authenticated API + tool-call gating | [#257](https://github.com/link-assistant/formal-ai/issues/257) |
| E13 | Network visualization + trace links on every surface | [#258](https://github.com/link-assistant/formal-ai/issues/258) |
| E14 | Natural-language skill compilation | [#259](https://github.com/link-assistant/formal-ai/issues/259) |

> See `docs/case-studies/issue-244/proposed-issues.md` for the full body of each
> planning issue.
