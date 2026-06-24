# Issue 559 Critical Review

**Historical status:** this review records the pre-implementation critical check
completed on 2026-06-23. PR #560 now implements the R330-R344 artifacts and the
live registry-backed dispatcher; see
[implementation-results.md](implementation-results.md) for the current state.
The "absent" rows below are retained as the exact gaps this PR closed or
scoped, not as open work for this branch.

This document is the "critical check of everything" the PR feedback requested. It
lists every imprecision found in the first-session planning artifacts, the
corrected fact with a `file:line` reference verified on 2026-06-23, and where the
correction is now reflected. It also gives a precise inventory of what is
**already built** versus **genuinely absent**, so the plan only invests effort in
the missing pieces.

## Method

Each claim below was checked directly against the working tree. References use
`path:line` form. "Verified absent" means a targeted search (`grep -rni`) over
`src/`, `desktop/`, and `vscode/` returned zero relevant hits.

## Corrections To First-Session Artifacts

### CR1 — Wrong lexicon path

- **First-session claim:** the lexicon lives in `data/meanings.lino`.
- **Fact:** there is no `data/meanings.lino`. The lexicon hub is
  `data/seed/meanings.lino` plus 34 `data/seed/meanings-*.lino` shards, embedded
  via `include_str!` in `src/seed/embedded.rs`.
- **Impact:** any phase that adds registry/frame data must target `data/seed/`
  (or `data/meta/`) to remain embedded in the binary.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md)
  ("Formalization And Routing", precision note).

### CR2 — Wrong location for `IntentFormalization`

- **First-session claim:** `IntentFormalization` lives under `src/concepts.rs` or
  `src/translation/formalization.rs`.
- **Fact:** `IntentFormalization` is declared at `src/intent_formalization.rs:48`.
  `src/concepts.rs` holds concept-lookup types only;
  `src/translation/formalization.rs:48` holds the separate
  `FormalizationCandidate`.
- **Impact:** the "evolve the meaning record" work targets
  `src/intent_formalization.rs`, not `concepts.rs`.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md),
  [alignment.md](alignment.md) (canonical mapping).

### CR3 — Non-existent handler names

- **First-session claim:** `SPECIALIZED_HANDLERS` includes "setup" and "UI"
  handlers.
- **Fact:** `SPECIALIZED_HANDLERS` (`src/solver_dispatch.rs:120`) has exactly 50
  entries; "setup" and "UI" are not among them. The real ordered keys are listed
  in [architecture-inventory.md](architecture-inventory.md) ("Specialized
  Handler Dispatch").
- **Impact:** the registry migration must enumerate the real 50 keys.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md).

### CR4 — "Web search and rerank are missing"

- **First-session claim (implied):** the solver lacks web search and reranking,
  so the evidence pipeline builds them from scratch.
- **Fact:** `src/web_search_core.rs` already provides a 33-provider registry
  (`WEB_SEARCH_PROVIDER_REGISTRY:90`), a 6-provider CORS-live subset
  (`WEB_SEARCH_PROVIDERS:327`), and Reciprocal Rank Fusion
  (`reciprocal_rank_fusion:396`, `WEB_SEARCH_RRF_K=60` at `:33`). The browser
  worker performs real search + RRF (`formal_ai_worker.js` ~`:35736`, ~`:35880`).
- **Impact:** the evidence pipeline *reuses* search + RRF and only adds the truly
  missing stages (crawl, extract, compare, hypothesize) plus non-CORS providers.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md)
  ("Existing Web Search, Rerank, And Fetch"),
  [evidence-pipeline.md](evidence-pipeline.md).

### CR5 — Conflating two "override" concepts

- **First-session claim:** "overrides" participate in routing.
- **Fact:** there are two unrelated concepts. (a) `data/overrides/` is a
  decorate-only **grounding-data** layer (one real file, `Q131560.lino`),
  consumed by `resolve(cache, override)`; it does not route. (b)
  `try_contextual_override` (`src/solver_dispatch.rs:84-111`) is a **code routing
  helper** that supplies extra arguments to five handlers (`proof_request`,
  `meta_explanation`, `numeric_list`, `shell_command_transform`,
  `text_manipulation`).
- **Impact:** the routing migration must not assume `data/overrides/`
  participates in routing; "preserve overrides" means preserve the grounding
  layer.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md)
  ("Specialized Handler Dispatch", "Data, Cache, Overrides, And Meanings").

### CR6 — Conflating on-disk and in-memory caches

- **First-session claim:** "caches" treated as one thing.
- **Fact:** on-disk caches are paired `.json`+`.lino` under `data/cache/`
  (wikidata/wiktionary/wordnet). In-memory caches are distinct Rust structures:
  the intent-formalization cache (`src/intent_formalization.rs`), the
  probability/replay store (`src/probability.rs`), and the append-only event log
  (`src/event_log.rs`).
- **Impact:** "preserve caches" must distinguish the two; a frame trace touches
  the event log (in-memory), not `data/cache/`.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md)
  ("Data, Cache, Overrides, And Meanings", precision note).

### CR7 — Loop ordering imprecision

- **First-session claim:** a simplified loop ordering.
- **Fact:** in `solve_with_history_probability_store_and_intent_cache`
  (`src/solver.rs:411-653`), local search (`search:local`, `:478`) runs **before**
  decomposition (`:480-483`), and the write-program rescue (`:525`) runs
  **before** synthesis (`:536`); concrete `WriteProgram` rules skip specialized
  handlers (`:551`).
- **Impact:** any registry selector that replaces direct dispatch must reproduce
  this order exactly to preserve behavior.
- **Fixed in:** [architecture-inventory.md](architecture-inventory.md)
  ("Current Universal Solver Loop", precision note),
  [recursive-core.md](recursive-core.md).

### CR8 — Decomposition over-described

- **First-session claim (implied):** decomposition is already a general recursive
  splitter.
- **Fact:** `UniversalSolver::decompose` (R74) is shallow conjunction splitting
  only — it splits on cues like `and`, `with tests`, `with benchmarks` — bounded
  by `SolverConfig::max_decomposition_depth`. It is not a general
  split-until-atomic recursion.
- **Impact:** the recursive core is a real extension of `decompose`, not a
  relabel of it; the gap is genuine.
- **Fixed in:** [recursive-core.md](recursive-core.md),
  [architecture-inventory.md](architecture-inventory.md) ("Follow-Up Feedback").

### CR9 — Vocabulary not anchored to canonical terms

- **First-session claim:** `ProblemFrame`/`Need`/`WorkUnit` introduced as new
  primitives.
- **Fact:** the canonical terms are impulse, requirement, sub-impulse, candidate,
  validation, the 11-step loop, and `SolverConfig` (`VISION.md:84-116`,
  `REQUIREMENTS.md` R72/R74/R157/R158). The new names must map onto these.
- **Impact:** see the canonical mapping in [alignment.md](alignment.md) (C1).
- **Fixed in:** [alignment.md](alignment.md).

### CR10 — Self-modification scope unbounded

- **First-session claim:** "the algorithm can modify itself" stated without the
  governing constraint.
- **Fact:** the only sanctioned path is `docs/design/self-improvement-loop.md`
  (proposal-only, gated by verification + benchmark + human review; never
  auto-appends to `data/seed/`). `NON-GOALS.md` forbids hidden autonomy.
- **Impact:** any self-improvement extension reuses this gate; issue 559 adds no
  autonomy.
- **Fixed in:** [alignment.md](alignment.md) (C3),
  [recursive-core.md](recursive-core.md).

### CR11 — External doctrine cited as repo doctrine

- **First-session claim:** R23 anchored on meta-theory "point-like/relation-like."
- **Fact:** that phrasing appears zero times in canonical repo docs; the canonical
  anchor is `VISION.md:44` ("Doublet links are the primitive storage model").
- **Impact:** R23 re-anchored to VISION; meta-theory stays a cited influence.
- **Fixed in:** [alignment.md](alignment.md) (C7),
  [requirements.md](requirements.md) (R23 source line).

### CR12 — CI gates omitted from verification

- **First-session claim:** verification matrix without the repo's hard gates.
- **Fact:** the repo enforces total reference closure
  (`scripts/audit-total-closure.py`, `tests/unit/total_closure.rs`), no-hardcoded
  natural language (four gates), requirement traceability
  (`tests/unit/docs_requirements.rs`), and the loop-event compatibility guard
  (`tests/unit/specification/reasoning_loop.rs:44`).
- **Impact:** the verification matrix now lists all of these.
- **Fixed in:** [alignment.md](alignment.md) (C6), [solution-plan.md](solution-plan.md).

## Pre-Implementation Built Versus Absent

This inventory keeps the plan honest about scope. The first columns preserve the
pre-implementation finding; the action column records the PR #560 outcome where
this branch changed the status.

| Capability | Status | Evidence | Issue-559 action |
| --- | --- | --- | --- |
| 11-step universal loop | Built | `src/solver.rs:411-653`; `VISION.md:84`; ROADMAP Pillar 2 | Make per-step state explicit; do not change shape |
| Formalized meaning record | Built (partial) | `IntentFormalization` (`src/intent_formalization.rs:48`); R157 | Extend into the explicit `ProblemFrame` |
| Intent routing from data | Built (partial) | `route_for_prompt` (`:342`); `data/seed/intent-routing.lino`; ROADMAP Pillar 20 | Generalize to method registry |
| Specialized handler dispatch | Built | `SPECIALIZED_HANDLERS` (`src/solver_dispatch.rs:120`), 50 entries | Wrap as registry methods; keep callable |
| Contextual override helper | Built | `try_contextual_override` (`:84-111`) | Preserve; model as method preconditions |
| Conjunction decomposition | Built (shallow) | `UniversalSolver::decompose`; R74 | Generalize to recursive split-until-atomic |
| Recursion depth knob | Built | `SolverConfig::max_decomposition_depth` | Reuse; add atomicity knob to config first |
| Multi-provider web search | Built | `WEB_SEARCH_PROVIDER_REGISTRY` (`src/web_search_core.rs:90`), 33 providers | Reuse; wire into evidence pipeline |
| Reciprocal Rank Fusion | Built | `reciprocal_rank_fusion` (`:396`), `WEB_SEARCH_RRF_K=60` (`:33`) | Reuse in rerank stage |
| Real network search/fetch | Built (browser) | `formal_ai_worker.js` ~`:35736`, `tryFetch` ~`:34849` | Reuse; add non-CORS seam |
| Source cache + provenance | Built | `data/cache/`; R67 (`source:`/`fetched_at`/`sha256`/`cache_hit`) | Reuse as evidence policy backing |
| Self-improvement proposal loop | Built | `src/self_improvement.rs:166`; `docs/design/self-improvement-loop.md` | Reuse the existing proposal-only gate |
| Single-skill compiler | Built (partial) | `src/skill_compiler.rs` | Extend toward a skill registry |
| Grounded recipe discipline | Built | `data/meta/*-recipe.lino`; `meta_algorithm.rs`; `agentic_meta_algorithm.rs` | Add a third general recipe |
| Five-rule substitution ladder | Built | `ARCHITECTURE.md` §9 (`:643`) | Use as registry method kinds |
| Explicit `ProblemFrame` type/event/schema | **Absent** | 0 hits for `ProblemFrame` in `src/` | Built in PR #560 via `src/meta_frame.rs` |
| Recursive `WorkUnit` with parent/child links | **Absent** | 0 hits for `WorkUnit`/`work_unit` in `src/` | Built in PR #560 via `src/meta_frame.rs` |
| Method registry covering all handlers | **Absent** | no registry indexing the 50 handlers + stdlib + skills | Built in PR #560 via `MethodRegistry::from_dispatch` and `meta_method_dispatch::try_dispatch`; skill promotion remains ledgered separately |
| Need-satisfaction ledger | **Absent** | no per-need satisfied/deferred/blocked tracking | Built in PR #560 via `NeedLedger` events |
| Crawl / full-content extraction | **Absent** | 0 hits for `crawl` in `src/`, `desktop/`, `vscode/` | New (evidence pipeline) |
| Live non-CORS providers (Google/Bing/Brave) | **Absent (live)** | registered but `cors_readable:false` | New (server/desktop fetch seam) |
| Expand→crawl→extract→compare loop | **Absent** | search+RRF exist; downstream stages do not | New (evidence pipeline) |
| Per-leaf method-selection auditability | **Absent** | BATTERY baselines exist but no selection trace | Built as the registry's `selection_mode` trace; the interim parity certificate that de-risked the migration was removed once the registry became the sole authority |
| General meta-algorithm recipe | **Absent** | only two specific recipes exist | Built in `data/meta/recursive-core-recipe.lino` and regenerated artifacts |

## Residual Risks Found During Review

1. **Worker parity is the weak flank.** ~30 `experiments/*-parity.mjs` exist but
   most are not in CI; only `issue-513-sync-worker-terminal.mjs --check` runs
   in-suite. Any shared-logic change in issue 559 must add a wired parity check,
   or it can silently diverge from the Rust canonical implementation
   (`ARCHITECTURE.md` §10.2).
2. **The load-bearing compat guard only tests arithmetic.**
   `specialized_handlers_still_publish_loop_events` must be widened before the
   registry migration, or "behavior preserved" is only proven for one family.
3. **Total-closure is strict.** Any new `.lino` data with an unresolved reference
   fails `tests/unit/total_closure.rs`. New registry/frame data must be authored
   closed.
4. **`.rs` 1000-line cap and `.lino` 1500-line cap are enforced**
   (`scripts/check-file-size.rs`). New code/data must respect them; large frame
   traces may need sharding (and `links-notation#197` streaming becomes relevant
   only then).
