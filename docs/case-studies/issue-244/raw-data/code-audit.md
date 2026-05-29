# Code Audit — Issue #244 (Actual State Of `src/`, Seed, And Tests)

This audit is the ground truth behind `../README.md`, `../../../ROADMAP.md`, and
`../proposed-issues.md`. It records what is **actually implemented** today (commit
`67a9fc5`), so the plan does not overstate the system. Line counts are from
`wc -l`; symbol locations are `file:line` at audit time.

## 1. The universal solver loop

- `src/solver.rs` (855 lines) documents and runs the **11-step loop** in its
  module comment (`src/solver.rs:1`): impulse → formalization → context →
  history → decomposition → TDD validation → synthesis → combination →
  verification → simplification → documentation. The loop is **deterministic**
  for a given `SolverConfig` + impulse, with "random guessing" seeded from the
  content-addressed impulse id.
- **Inner routing is still keyword/intent driven.** `solve_with_history()`
  calls `handle_specialized_pattern()` (`src/solver.rs:556`), which walks an
  ordered `SPECIALIZED_HANDLERS` dispatch table (`src/solver.rs:412`) of **35
  free-function handlers** plus 3 special-cased handlers run before the table
  (`behavior_rules`, `feature_capability`, `playwright_script`). The first
  handler that returns `Some` claims the impulse. Each handler is selected by
  seed keyword / phrase / token / combo matching, not by a single
  formalize → search → decompose → candidates → validate → select pass.
- **Consequence for the vision:** the loop *shape* exists and is logged, but the
  promise that "every prompt walks the same universal algorithm" (VISION.md
  "Universal Problem-Solving Algorithm") is not yet true at the routing layer.
  This is the core of epic **E2**.

## 2. SolverConfig knobs

`SolverConfig` (`src/solver.rs:153`) already carries every documented knob:

| Field | Line | `FORMAL_AI_*` override |
| --- | --- | --- |
| `guess_probability` | 160 | `FORMAL_AI_GUESS_PROBABILITY` |
| `context_sensitivity` | 170 | (config only) |
| `questioning_rigor` | 172 | (config only) |
| `temperature` | 174 | `FORMAL_AI_TEMPERATURE` |
| `max_decomposition_depth` | 176 | (config only) |
| `agent_mode` | 178 | `FORMAL_AI_AGENT_MODE` |
| `diagnostic_mode` | 180 | `FORMAL_AI_DIAGNOSTIC_MODE` |
| `offline` | 182 | `FORMAL_AI_OFFLINE` |
| `cache_ttl_seconds` | 184 | `FORMAL_AI_CACHE_TTL_SECONDS` |

`temperature` exists but there is **no softmax / ε-comparison helper** that turns
candidate formalization scores into a selection (ARCHITECTURE.md §16.2 — epic
**E4**).

## 3. Event log and durable store

- `src/event_log.rs` (276 lines): an **in-process append-only event log** with
  content-addressed (FNV-1a) ids and the documented event kinds (`impulse:`,
  `search:local`, `search:external`, `sub_impulse:`, `candidate:`,
  `validation:`, `trace:`, `cache_hit:`, `source:`, `policy:`, `agent_mode:`,
  `error:`). It is rebuilt per request; it is not the durable store.
- `src/memory.rs` (535 lines) + `src/memory/bundle.rs` (580 lines): the durable
  store is a **custom `MemoryStore` backed by `.lino`**, not
  `doublets-rs` / `doublets-web`. ARCHITECTURE.md §4.2 describes the durable
  doublets store as wrapped behind a trait with the crate dependency **not yet
  pulled in** (§16.3). This split store is the core of epic **E1**.

## 4. Formalization

- `src/concepts.rs` (486 lines): formalization is **alias based** —
  `extract_concept_query` / `lookup_concept_query` resolve known aliases to
  concept records. Full **Wikidata P-id (verbs/properties) / Q-id
  (nouns/items) extraction over arbitrary prompts** is not implemented
  (ARCHITECTURE.md §5, §16.1 — epic **E3**).

## 5. Translation pipeline

- `src/translation/` is a **real pipeline**: `pipeline.rs` (805 lines) runs
  `formalize → meaning → deformalize → match_source_formatting`;
  `wiktionary.rs` (829 lines) parses per-language entries; `wikidata.rs`
  (372 lines) issues SPARQL; `cache.rs` (907 lines) caches lookups.
- The **link-native meaning-id invariants** (synonyms share one meaning id;
  traces include the intermediate meaning record; untranslatable terms are
  flagged) are tracked as `#[ignore]` tests, not yet enforced — epic **E6**.

## 6. Proof / formal reasoning

- `src/proof_engine/` is a **classical-theorem registry**: `mod.rs` (713 lines),
  `library.rs` (766 lines), `arithmetic.rs` (309 lines), `presenter.rs`
  (479 lines). It recognizes a fixed set of theorems and presents a formatted
  proof. There is **no general decision procedure**; "formal reasoning that
  covers all test cases and much more" (issue Q9) points at integrating
  `relative-meta-logic` / an SMT backend — epic **E8**.

## 7. Agent mode

- Agent mode is **guarded but never executed**: `SolverConfig.agent_mode`
  defaults off and gates behavior, but chat never runs user code. There is no
  sandbox, action log, confirmation flow, time budget, or secret guard yet
  (epic **E11**). This matches NON-GOALS.md ("Agent mode is not intended for
  unsafe use … without isolation").

## 8. The `#[ignore]` "tracked requirement" backlog

The precise, machine-checkable backlog is **69 `#[ignore]`-tagged tests** under
`tests/unit/specification/`. Each is annotated `tracked requirement: …`. They are
the acceptance criteria for the planning issues — every epic in
`../proposed-issues.md` names the exact tests it must graduate out of `#[ignore]`.

| Spec file | Ignored | Theme |
| --- | --- | --- |
| `reasoning_loop.rs` | 11 | Universal loop steps 1–9, termination, confidence |
| `links_network.rs` | 10 | Doublet reduction, type chains, append-only, source/trace links, schema version, addressability, validation |
| `agent_isolation.rs` | 9 | Agent opt-in, sandbox, action log, failure traces, confirmation, time budget, secret guard, revocation |
| `transparent_state.rs` | 8 | Network query, "what do you know about X", no leak, diagnostic opt-in, "why", retraction, export, "list my facts" |
| `source_cache.rs` | 8 | Source URL, `fetched_at`, TTL refresh, cache hit, content hash, conflict surfacing, explicit flush, offline |
| `translation_via_links.rs` | 7 | Meaning-id preservation, target surface, synonyms share meaning, language tags, intermediate meaning, code translation, untranslatable flag |
| `code_generation.rs` | 6 | Top-10 languages, execution links, isolation level, algorithm+tests, program translation, failure traces |
| `chat_surface.rs` | 6 | Refuse unbounded, declare execution status, diagnostics off, impulse recorded, trace link, extend-network path |
| `openai_compatibility.rs` | 2 | Bearer auth, refuse tool call without agent mode |
| `telegram_surface.rs` | 1 | Tap-to-inspect trace link |
| `network_visualization.rs` | 1 | Graph beside chat never blocks replies |
| **Total** | **69** | |

(The other spec files — `capabilities.rs`, `multilingual.rs`,
`prompt_variations.rs`, `reasoning_paths.rs`, `definition_fusion.rs`,
`issue_146.rs`, `calculator_delegation.rs`, `project_lookups.rs`,
`summarization_pipeline.rs` — have **0** ignored tests; they are the green
regression floor that no epic may break, per issue Q12.)

## 9. Architecture open questions (`ARCHITECTURE.md` §16)

1. Full Wikidata P/Q-id formalization — epic **E3**.
2. Softmax temperature helper — epic **E4**.
3. `doublets-rs` backend dependency — epic **E1**.
4. Natural-language-skill compiler — epic **E14**.

## 10. Documentation drift found and corrected in this PR

- `ARCHITECTURE.md:788` referenced `REQUIREMENTS.md` as "R1 … R149"; the matrix
  actually runs to **R230** across issue sections up to `## Issue #196`. Updated.
- `REQUIREMENTS.md` had **no Issue #244 section**; one is added by this PR.
- There was **no consolidated implementation-progress tracker**; `ROADMAP.md`
  is added to satisfy issue Q2/Q3 ("update documentation to fully track
  progress … in sync with the actual state of the code").
