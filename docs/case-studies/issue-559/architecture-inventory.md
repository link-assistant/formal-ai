# Issue 559 Architecture Inventory

This inventory documents the current architecture before implementation
planning, as requested in the issue. Every claim is grounded in a concrete
`file:line` reference verified against the working tree on 2026-06-23. Where the
first-session draft of this document was imprecise, the correction is called out
in [critical-review.md](critical-review.md) and applied here.

## Documents Read

- `README.md`
- `ARCHITECTURE.md` (1076 lines)
- `REQUIREMENTS.md` (813 lines, requirements R1–R329)
- `VISION.md`
- `GOALS.md`
- `NON-GOALS.md`
- `ROADMAP.md`
- `docs/meta-algorithm.md`
- `docs/design/no-hardcoded-natural-language.md`
- `docs/design/self-improvement-loop.md`
- `docs/design/rule-synthesis.md` (the only three files under `docs/design/`)
- `CONTRIBUTING.md`
- Existing case studies, especially `docs/case-studies/issue-433`,
  `docs/case-studies/issue-468`, `docs/case-studies/issue-412`, and
  `docs/case-studies/issue-554`
- Follow-up PR feedback on
  <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783154352>
  and <https://github.com/link-assistant/formal-ai/pull/560#issuecomment-4783640128>

## Current Universal Solver Loop

`src/solver.rs` is the main runtime entry point. `UniversalSolver` is declared
at `src/solver.rs:349`. The public entry `solve` (`:373`) delegates through
`solve_with_history` (`:382`) and
`solve_with_history_and_probability_store` (`:396`) to the real implementation
`solve_with_history_probability_store_and_intent_cache` (`:411`). The body at
`:411`–`653` performs, in order:

1. Record the prompt and runtime context; detect language.
2. Apply probability replay when enabled.
3. Produce formalization candidates and select/cache an
   `IntentFormalization`.
4. Run local search (`search:local`, `:478`) **before** decomposition
   (`:480`–`483`).
5. Try the write-program rescue path (`:525`) **before** synthesis (`:536`).
6. Run specialized handlers via `handle_specialized_pattern`, except that
   concrete `WriteProgram` rules skip them (`:551`).
7. Apply policy and unknown-reasoning fallbacks.
8. Record candidate, validation, simplification, and answer-projection events.

This means the repo already has a traceable universal envelope, which
`ARCHITECTURE.md` §7 (line 516) calls the "Universal Problem Solver" and
`REQUIREMENTS.md` R72 (`:110`) requires as an 11-step loop that runs "for every
request without branching by domain". Issue 559 should generalize what happens
*inside* that envelope — specifically method selection — rather than add a new
top-level branch.

> Precision note: the order above (local search before decomposition,
> write-program before synthesis) was verified against `src/solver.rs` and
> differs slightly from the simplified ordering in the first draft of this
> document. The corrected order matters for the migration phases because any
> registry selector must reproduce it exactly.

## Meta Algorithm Assets

`docs/meta-algorithm.md` currently describes two grounded recipes, and
`data/meta/` contains **exactly two files**:

- Procedural how-to recipe: `data/meta/procedural-howto-recipe.lino`
  (198 lines), grounded by `tests/unit/specification/meta_algorithm.rs`
  (275 lines, issue #444).
- Agentic coding recipe: `data/meta/agentic-coding-recipe.lino` (244 lines),
  grounded by `tests/unit/specification/agentic_meta_algorithm.rs` (337 lines,
  issue #468).

Both are useful precedents because they describe behavior in data and then
ground it through tests: the test loads the `.lino` recipe and asserts the live
source still matches every record (`meta_algorithm.rs:10-13`). If recipe and
code drift, CI fails. The gap is that they are still **specific** algorithms
(one chat-intent family, one agentic loop), generalized only by *copying a
recipe per topic* (`docs/meta-algorithm.md`, "Generalising to a new topic").
There is no single general problem-frame recipe that every request passes
through.

Issue #433 is also directly relevant. It audits each specialized handler
recognizer as fixed enumeration, hybrid, or compositional, then demonstrates how
`numeric_list` can be reconstructed from the issue #423 meta-algorithm
primitives. That makes the current missing piece clearer: the repository has
examples of class-level construction recipes, but no single registry where every
method advertises its preconditions, required evidence, validation policy, and
executable hook.

Issue #412 is the roadmap's named next step: `ROADMAP.md` Pillar 7 lists a
"task-agnostic meta-builder ('algorithm that builds algorithms', R7)" tracked in
`docs/case-studies/issue-412`. Issue 559 should be positioned as delivering that
residual rather than as an unrelated initiative.

## Formalization And Routing

Relevant files (corrected locations):

- `src/intent_formalization.rs` — the `IntentFormalization` struct is declared
  at `:48`–`60` (fields: `impulse_id`, `source_text`, `normalized_text`,
  `language`, `kind`, `knowns`, `relevants`, `parameters`, `route`,
  `response_link`). This is the current carrier of "the formalized meaning of a
  prompt" and is the natural place to evolve toward a general problem frame.
- `src/translation/formalization.rs:48-55` — `FormalizationCandidate`
  (`slots: Vec<FormalizationSlot>`), the translation-side formalizer.
- `src/concepts.rs` — concept-lookup types only (this is **not** where
  `IntentFormalization` lives; the first draft of this document was wrong on
  that point).
- `data/seed/*.lino` — 66 seed files, including the lexicon hub
  `data/seed/meanings.lino` plus 34 `data/seed/meanings-*.lino` shards,
  `data/seed/roles.lino`, `data/seed/intent-routing.lino`,
  `data/seed/sources-registry.lino`, and `data/seed/prompt-patterns.lino`
  (1070 lines).

> Precision note: there is **no** `data/meanings.lino`. The lexicon lives under
> `data/seed/`. The first draft referenced the wrong path; later phases that add
> registry data must target `data/seed/` (or `data/meta/`) so the
> `include_str!` embedding in `src/seed/embedded.rs` continues to work.

Routing today is mostly data-driven but retains hardcoded English cue
recognizers:

- `route_for_prompt` (`src/intent_formalization.rs:342-357`) is mostly
  data-driven via `seed::intent_routing()` with one hardcoded write-program
  branch.
- `ordered_handler_names` (`:311-334`) orders handlers without natural-language
  predicates.
- `append_prompt_relevants` (`:718-800`) still derives relevants from Rust
  string predicates — e.g. `["search","google","find"]` (`:729`) and
  `["build","create","implement","develop"]` (`:759`).
- `looks_like_text_manipulation` (`:832-851`) embeds a fully hardcoded English
  operations list.

These hardcoded cue lists are exactly what `docs/design/no-hardcoded-natural-language.md`
("Natural language is data, never a string literal in the engine") and
`REQUIREMENTS.md` R97 (`:156`, externalize hardcoded surface constants) tell us
to move into seed data.

## Specialized Handler Dispatch

Relevant file:

- `src/solver_dispatch.rs`

`SPECIALIZED_HANDLERS` is declared at `src/solver_dispatch.rs:120` as
`pub const SPECIALIZED_HANDLERS: &[(&str, SpecializedHandler)]` with **exactly
50 entries**. The module doc (`:1-6`) and the precedence comment (`:113-119`)
state it is the single source of truth for handler precedence and that the first
matching handler wins. The real, ordered handler keys are:

```
http_fetch, url_navigate, web_search, research_comparison_table,
docs_method_explanation, procedural_how_to, procedural_how_to_followup,
conversation_memory, software_project_followup, summarization,
text_manipulation, brainstorming, conversation_topic, fact_lookup,
coreference, roleplay, translation, capabilities, calendar_reasoning,
calendar_create_event, compound_interest, numeric_list,
shell_command_transform, number_constraint_reasoning, arithmetic,
javascript_execution, definition_merge, concept_lookup, who_is, how_it_works,
meta_explanation, network_query, execution_failure, installation_conversion,
write_script, program_synthesis, document_generation_plan, software_project,
algorithm, source_refresh, source_conflict, clarification,
punctuation_only_prompt, ill_formed, physical_action_question, kupi_slona,
shell_refusal, proof_request, opinion_question, incompatible_units
```

> Precision note: the first draft listed "setup" and "UI" handlers. Those do
> not exist. The 50 keys above are the actual table; the migration inventory in
> Phase 3 must cover exactly these.

A separate code helper, `try_contextual_override`
(`src/solver_dispatch.rs:84-111`), supplies extra arguments for five handlers
that need context beyond the prompt: `proof_request`, `meta_explanation`,
`numeric_list`, `shell_command_transform`, and `text_manipulation`. This is a
**routing helper in code**, unrelated to the grounding-data layer under
`data/overrides/` (see "Data, Cache, Overrides, And Meanings" below). The two
"override" concepts must not be conflated during migration.

Relevant runtime:

- `UniversalSolver::handle_specialized_pattern` (`src/solver.rs:655-767`) is the
  Rust control plane; it runs pre-table handlers (`:687-702`) and then consults
  `ordered_handler_names` (`:707-710`).

The current system already lets intent formalization influence handler order by
placing relevant handler names first. However, the executable control plane is
still the handler table plus Rust predicates. `ROADMAP.md` Pillar 20 captures
this precisely: routing is "by formalized intent, not a fixed catalogue"
(Built), with the caveat "`SPECIALIZED_HANDLERS` remain as a precedence table
behind the formalized router." Issue 559 targets that residual.

## Existing Web Search, Rerank, And Fetch

A correction to the first draft: the repository **already has** a multi-provider
web search core with reranking. Issue 559 should treat these as building blocks,
not as missing pieces.

- `src/web_search_core.rs` is a `no_std`, network-free, symbolic core.
  - `WEB_SEARCH_PROVIDER_REGISTRY` (`:90-315`) lists **33 providers** grouped by
    `ProviderCategory` (Search/Knowledge/Papers/Code), including `google`
    (`:99`, `cors_readable: false`), `bing`, `brave`, `duckduckgo`, plus
    Wikipedia/Wikidata/Wiktionary, arXiv, GitHub, and others.
  - The live in-browser subset `WEB_SEARCH_PROVIDERS` (`:327-334`) is the **6
    CORS-readable** ids only: `duckduckgo, internet-archive, wikipedia,
    wikidata, wiktionary, wikinews`.
  - Reciprocal Rank Fusion already exists:
    `reciprocal_rank_fusion(entries, k)` (`:396-438`), with
    `WEB_SEARCH_RRF_K = 60` (`:33`).
- `src/solver_handlers/web_requests.rs` re-exports the core consts (`:147`,
  `:153`) and implements `try_web_search` (`:155-276`), `try_http_fetch`
  (`:22-85`), and `try_url_navigate` (`:116-140`). These Rust handlers are
  **descriptive**: they log the plan (e.g. `web_search:combined rrf:k=60`) and
  return prose describing what the browser will do. They do not themselves
  perform network I/O.
- The real network engine lives in the browser worker
  `src/web/formal_ai_worker.js`: live provider dispatch and concurrency
  (~`:35736`–`:36254`), `reciprocalRankFusion` (~`:35880`) calling the WASM
  export `wasm.web_search_fuse`, and `tryFetch` (~`:34849`–`:34937`) which does
  a real CORS GET truncated to 2000 bytes.
- The desktop app is an Electron wrapper that serves `src/web/` into a
  `BrowserWindow` (`desktop/main.cjs:200-258`); its only direct network
  primitive is a permission-gated `http_fetch` GET in
  `desktop/lib/tool-router.cjs:92-115`. There is **no** dedicated npm
  web-search dependency; `agent-commander` is the agent-execution provider, not
  a search library.

What is genuinely **absent** (and therefore in scope for issue 559's evidence
pipeline): crawling/full-content extraction of reranked result pages
(`grep -rni crawl` over `src/`, `desktop/`, `vscode/` returns 0 hits), live
non-CORS providers such as Google wired through a server/desktop fetch seam, and
a general expand→search→rerank→crawl→extract→compare loop reusable by any
method. See [evidence-pipeline.md](evidence-pipeline.md).

## Existing Skill And Self-Improvement Building Blocks

- `src/skill_compiler.rs` is a single-skill compiler (it compiles one skill
  description), not a registry/library. It is the seed of the "skills as data"
  direction but does not yet index many skills with applicability metadata.
- `src/self_improvement.rs` implements the proposal-only loop: `UnknownTrace`
  (`:24`), `learn_rules_from_unknown_traces` (`:166`), and `LearnedRuleProposal`
  (`:188`). It proposes Links Notation rules from accumulated unknown traces and
  blocks adoption behind verification and benchmark gates
  (`docs/design/self-improvement-loop.md`). Issue 559 reuses this exact gate for
  any self-modification (Phase 9) — nothing in 559 may bypass it.
- `ARCHITECTURE.md` §9 ("Transformation and Substitution Rules", line 643)
  documents a five-rule ladder from pure data rules → Rust handlers → sandboxed
  JS → dynamically compiled code stored as data → natural-language skills. This
  is the strongest existing on-ramp for "algorithms as data" and is the
  conceptual home for the method/skill registry.

## Data, Cache, Overrides, And Meanings

The issue requires preserving this architecture. The real layout (`data/` =
11 directories, 1386 `.lino` files) is:

- `data/seed/` (66 files) — the lexicon and routing data, embedded into the
  binary via `include_str!` in `src/seed/embedded.rs`. Includes
  `meanings.lino` + 34 `meanings-*.lino`, `roles.lino`, `intent-routing.lino`,
  `sources-registry.lino`, `prompt-patterns.lino`.
- `data/meta/` (2 files) — the grounded recipes (above).
- `data/cache/` — paired `.json` + `.lino` source caches
  (wikidata / wiktionary / wordnet). This is the **on-disk** cache.
- `data/overrides/` — a decorate-only grounding layer (one real override,
  `Q131560.lino`) consumed by `resolve(cache, override)`. This is **grounding
  data**, not routing.
- `data/view/en/` (551 files) — generated views.
- `data/benchmarks/` — 4 ratchet suites.
- `data/parity/cross-runtime-synthesis.json` — Rust↔JS parity fixtures.

> Precision note: the in-memory caches are different artifacts and are **not**
> on-disk `.lino`: the intent-formalization cache (`src/intent_formalization.rs`),
> the probability/replay store (`src/probability.rs`), and the append-only event
> log (`src/event_log.rs`). Any plan that talks about "preserving caches" must
> distinguish the on-disk source cache (`data/cache/`) from these in-memory
> Rust structures.

The plan must migrate **control** data into these surfaces (preferring
`data/seed/` and `data/meta/`) instead of replacing them, and any new `.lino`
data must pass the total reference-closure gate (`scripts/audit-total-closure.py`,
must report `unresolved_distinct: 0`) and the worker-mirror `--check`.

## Testing Surfaces

Cargo wires three test binaries — `unit`, `integration`, `source`
(`Cargo.toml:73-83`). `tests/source/` is a vendored mirror of `src/` used to
test private functions. Relevant suites:

- `tests/unit/specification/reasoning_loop.rs` — contains the most load-bearing
  compatibility guard, `specialized_handlers_still_publish_loop_events`
  (`:44-70`), which asserts specialized handler answers still publish candidate,
  validation, and simplification evidence. Today it only exercises arithmetic;
  Phase 1 should widen it. A migration can keep handlers but must route them
  through the universal evidence model.
- `tests/unit/specification/meta_algorithm.rs` (#444) and
  `tests/unit/specification/agentic_meta_algorithm.rs` (#468) — ground the
  `data/meta/*-recipe.lino` recipes against live source via a hand-rolled
  `.lino` parser (`Record`/`parse_record`).
- `tests/unit/prompt_variations.rs` (944 lines, #103) — 5–10 variations per
  case across EN/RU/HI/ZH, with helpers `assert_intent_for_each`,
  `assert_language_for_each`, `assert_answer_contains_for_each`
  (`REQUIREMENTS.md` R129/R132).
- `tests/unit/total_closure.rs` — shells to `python3
  scripts/audit-total-closure.py`.
- `tests/unit/docs_requirements.rs` (+ `docs_requirements/benchmarks.rs`,
  `docs_requirements_issue_451.rs`, `docs_requirements_issue_468.rs`) — assert
  REQUIREMENTS.md literally contains each `| R<n> ` marker, enforcing
  requirement→test traceability. Issue 559 will add a
  `issue_559_..._are_traceable` test pinning new rows.
- Benchmark ratchets (`data/benchmarks/`) and frozen BATTERY baselines
  (e.g. `tests/unit/translation/mod/parity.rs`, a 93-row table) provide
  ready-made old/new comparison infrastructure.
- Worker parity is the **weak flank**: ~30 `experiments/*-parity.mjs` harnesses
  exist but most are not wired into CI (only
  `issue-513-sync-worker-terminal.mjs --check` runs in-suite, plus
  function-name presence checks). Any cross-runtime change in issue 559 must
  strengthen this flank.

## Existing Self-Improvement Design

`docs/design/self-improvement-loop.md` (issue #364) already defines the intended
safe path: unknown traces can suggest new Links Notation rules; suggestions are
not silently applied; generated changes must pass verification and benchmarks;
human review remains part of the loop. Issue 559 reuses this as the later
self-modification gate for changing the general meta algorithm.

## Follow-Up Feedback Applied To Architecture

The 2026-06-23 PR feedback adds architectural constraints not fully captured by
the initial first-session plan:

1. **Recursive solving is not just optional decomposition.** Today decomposition
   is shallow conjunction splitting in `UniversalSolver::decompose`
   (`REQUIREMENTS.md` R74: splits on `and`, `with tests`, `with benchmarks`),
   bounded by `SolverConfig::max_decomposition_depth`. There is no universal
   recursive work-unit model that keeps splitting until a unit is directly
   solvable by a method, library call, standard function, or reviewed skill.
2. **Skill accumulation is not a first-class control plane.** Existing handlers,
   examples, generated source, seed rules, `skill_compiler.rs` output, and
   standard-library functions can behave like reusable skills, but the
   repository does not index them with applicability, proof status, examples,
   negative examples, validation cost, and safe reuse boundaries.
3. **Fresh-data research is partly built but not general.** Multi-provider
   search and RRF exist (`src/web_search_core.rs`), but there is no general
   evidence pipeline that expands a prompt into terms/phrases/sentences/
   questions; searches multiple providers (including non-CORS); reranks;
   crawls; extracts; compares; and feeds hypothesis gaps back into the same
   recursive loop.
4. **Link-native modeling needs to be explicit.** Links Notation and the
   doublet store already use links (`VISION.md`: "Doublet links are the
   primitive storage model for this project"), but future docs should avoid a
   separate non-link ontology. Dependency, task, method, evidence, and sequence
   relations should be represented as links. The meta-theory "point-like /
   relation-like are both links" framing is an upstream reference, not yet repo
   doctrine; the plan re-anchors it to the doublet primitive.
5. **Dependency readiness is not gated.** The repo uses several
   organization-owned crates and packages, but the plan did not previously say
   which upstream capability must exist before each phase can begin. See
   [upstream-dependency-audit.md](upstream-dependency-audit.md).

## Architecture Gaps To Address

1. Method selection is not yet a first-class, data-described reasoning step.
2. Specialized handler precedence still lives in Rust ordering
   (`SPECIALIZED_HANDLERS`, `src/solver_dispatch.rs:120`).
3. Some natural-language cue recognition still lives in Rust code
   (`append_prompt_relevants`, `looks_like_text_manipulation`).
4. Big-task planning is documented for agentic coding (`data/meta/agentic-coding-recipe.lino`)
   but not represented as a general recursive task-link network for every large
   request.
5. Chat-mode fresh-data policy is not a unified evidence policy in the solver
   frame; web search/RRF exist but are handler-specific and crawl is absent.
6. Existing tests cover many behaviors, but they do not yet prove class-level
   parity for every historical handler family (the load-bearing guard only
   checks arithmetic today).
7. Meta recipes exist per capability; the project needs a general recipe (a
   third `data/meta/*-recipe.lino` grounded by a test) that can call those
   methods as data-described submethods.
8. There is no `ProblemFrame` type/event/`.lino` schema that every solver
   response emits (verified: 0 occurrences of `ProblemFrame` in `src/`). The
   closest analog is `IntentFormalization`.
9. There is no `WorkUnit`/`work_unit` recursive unit (verified: 0 occurrences in
   `src/`) recording parent/child links, atomicity, required evidence, selected
   skill, validation result, and composition result. The closest analog is the
   `sub_impulse` produced by `UniversalSolver::decompose`.
10. There is no method/skill registry covering all 50 `SPECIALIZED_HANDLERS`,
    the five contextual-override handlers, standard-library operations, repo
    helper functions, generated examples, and future learned rules.
11. There is no old-vs-new comparison mode for registry selection before
    replacing direct dispatch (though BATTERY baselines and prompt-variation
    harnesses provide the infrastructure to build one).
12. There is no uniform need-satisfaction ledger that forces the final answer to
    mark every detected question, requirement, constraint, and deferred item
    (`REQUIREMENTS.md` R158 already asks to "model the task as a graph of
    requirements and subtasks" — the ledger operationalizes this).
13. There is no general search/rerank/crawl/extract/evaluate loop for fresh-data
    chat questions (search + RRF exist; crawl/extract/compare do not).
14. There is no upstream dependency gate document that says when implementation
    must pause for `meta-language`, `links-notation`, `lino-objects-codec`,
    `doublets`, `link-calculator`, or `agent-commander` (now provided by
    [upstream-dependency-audit.md](upstream-dependency-audit.md)).

## Upstream Dependency Snapshot

The related organization dependency audit is captured in
[upstream-dependency-audit.md](upstream-dependency-audit.md). The important
architecture conclusion is:

- No upstream blocker prevents the next behavior-preserving phases:
  `ProblemFrame` trace, recursive work-unit tracing, method registry inventory,
  and old/new selection comparison.
- `meta-language` already advertises mutable link networks, source spans,
  lossless parse/reconstruction, generated-source rendering, snapshots,
  structural query/replace, substitutions, LiNo parsing, and cross-language
  reconstruction. Those capabilities are enough for the planned
  algorithm-as-data representation.
- `links-notation` has an existing streaming-parser issue
  (<https://github.com/link-foundation/links-notation/issues/197>). It matters
  for very large future frame exports but is not a blocker for the initial
  registry and frame data.
- `link-calculator`, `doublets`, `platform-mem`, `lino-objects-codec`,
  `lino-arguments`, `lino-i18n`, and `agent-commander` do not currently block
  the next implementation phases.

## Compatibility Constraints

- Preserve existing user-visible behavior unless a test and requirement
  explicitly approve a change.
- Preserve `.lino`, cache (on-disk `data/cache/` and in-memory stores),
  override (`data/overrides/` grounding layer), meanings (`data/seed/`), and
  source-cache architecture.
- Keep current specialized handlers callable during migration.
- Keep Rust and browser worker mirrors aligned (`ARCHITECTURE.md` §10.2: "The
  Rust pipeline is the canonical implementation").
- Add tests before changing routing behavior; new knobs go into `SolverConfig`
  first (`NON-GOALS.md`: "Bypassing `SolverConfig` for hard-coded behavior is
  not acceptable").
- Avoid direct adoption of a large external orchestration runtime as the core
  engine unless it can satisfy Rust, offline, deterministic, data-driven, and
  parity constraints.
