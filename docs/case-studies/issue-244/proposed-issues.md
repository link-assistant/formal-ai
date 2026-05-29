# Proposed Planning Issues — Issue #244

These are the full bodies of the planning issues created to "fully implement our
vision" (issue #244). They are drafted here first so the plan is reviewable in
one place, then opened in the repository. Each issue links back to #244 as its
parent and is labeled `enhancement`.

The 2026-05-25 batch (E1-E14) is now closed on `main`. The 2026-05-26
post-implementation audit opened a narrower follow-up batch (E15-E20) for the
requirements that remained partial after those merges, and that batch is now
closed too. A third-pass audit (acting on issue #244 feedback) opened the
reasoning-focused batch E21-E27 — **now also closed** by merged PRs #305-#311.
A fourth-pass audit (2026-05-27) found the remaining gap was the **generality of
the synthesis step** and opened the synthesis batch E28-E32 — **now also closed**
by merged PRs #319-#323; the synthesis step derives answers and the benchmark
suite passes 10/10. A fifth-pass audit (2026-05-29, PR #245 feedback) found the
remaining gap is **parity** ("all Rust and JavaScript logic in sync", "all
languages supported equally") and opened the parity batch E33-E34 — **now also
closed** by merged PRs #328-#329: the text-manipulation handler triggers from one
shared data-driven multilingual operation vocabulary, and the JS browser worker
derives the same synthesis/numeric/program/text answers as the Rust core. With
E1-E34 all merged, no vision-planning epic remains open for issue #244. The full
bodies of E28-E32 and E33-E34 are at the end of this file.

**Opened issues (2026-05-25):** E1 → [#246](https://github.com/link-assistant/formal-ai/issues/246),
E2 → [#247](https://github.com/link-assistant/formal-ai/issues/247),
E3 → [#248](https://github.com/link-assistant/formal-ai/issues/248),
E4 → [#249](https://github.com/link-assistant/formal-ai/issues/249),
E5 → [#250](https://github.com/link-assistant/formal-ai/issues/250),
E6 → [#251](https://github.com/link-assistant/formal-ai/issues/251),
E7 → [#252](https://github.com/link-assistant/formal-ai/issues/252),
E8 → [#253](https://github.com/link-assistant/formal-ai/issues/253),
E9 → [#254](https://github.com/link-assistant/formal-ai/issues/254),
E10 → [#255](https://github.com/link-assistant/formal-ai/issues/255),
E11 → [#256](https://github.com/link-assistant/formal-ai/issues/256),
E12 → [#257](https://github.com/link-assistant/formal-ai/issues/257),
E13 → [#258](https://github.com/link-assistant/formal-ai/issues/258),
E14 → [#259](https://github.com/link-assistant/formal-ai/issues/259).
See `ROADMAP.md` for current status and dependency notes.

**Opened issues (2026-05-26):** E15 → [#278](https://github.com/link-assistant/formal-ai/issues/278),
E16 → [#279](https://github.com/link-assistant/formal-ai/issues/279),
E17 → [#280](https://github.com/link-assistant/formal-ai/issues/280),
E18 → [#281](https://github.com/link-assistant/formal-ai/issues/281),
E19 → [#282](https://github.com/link-assistant/formal-ai/issues/282),
E20 → [#283](https://github.com/link-assistant/formal-ai/issues/283).

**Opened issues (2026-05-26, reasoning batch):** E21 → [#298](https://github.com/link-assistant/formal-ai/issues/298),
E22 → [#299](https://github.com/link-assistant/formal-ai/issues/299),
E23 → [#300](https://github.com/link-assistant/formal-ai/issues/300),
E24 → [#301](https://github.com/link-assistant/formal-ai/issues/301),
E25 → [#302](https://github.com/link-assistant/formal-ai/issues/302),
E26 → [#303](https://github.com/link-assistant/formal-ai/issues/303),
E27 → [#304](https://github.com/link-assistant/formal-ai/issues/304).
All closed by merged PRs #305-#311.

**Opened issues (2026-05-27, synthesis batch):** E28 → [#313](https://github.com/link-assistant/formal-ai/issues/313),
E29 → [#314](https://github.com/link-assistant/formal-ai/issues/314),
E30 → [#315](https://github.com/link-assistant/formal-ai/issues/315),
E31 → [#316](https://github.com/link-assistant/formal-ai/issues/316),
E32 → [#317](https://github.com/link-assistant/formal-ai/issues/317).

**Design rules that bind every epic** (from `../README.md`):

- **Foundation first (Q13).** E1 and E2 are blockers; the rest build on them.
- **Keep the regression floor (Q12).** No epic removes an already-supported
  behavior. The currently-green spec files (`capabilities.rs`, `multilingual.rs`,
  `prompt_variations.rs`, `reasoning_paths.rs`, `definition_fusion.rs`,
  `issue_146.rs`, `calculator_delegation.rs`, `project_lookups.rs`,
  `summarization_pipeline.rs`) must stay green; the first batch graduated
  tracked tests, and follow-up work must add or narrow tests instead of deleting
  passing ones.
- **Determinism and traceability (Q8).** Same prompt + same `SolverConfig` ⇒ same
  event log and answer; randomness seeded from the impulse hash; every answer
  carries an inspectable `trace:` pointer.
- **Web as cache, not teacher (Q10).** External knowledge is cached with
  provenance (E5); offline mode refuses lookups; nothing is learned into weights.

**Acceptance-criteria convention.** E1-E14 list the exact original `#[ignore]`
"tracked requirement" tests they had to graduate under
`tests/unit/specification/`. The post-implementation audit confirmed that zero
tracked ignored tests remain. E15-E20 list the smaller remaining requirements
found in stale deferred markers, architecture open questions, and partially
implemented requirement rows.

---

## E1 — Unified doublet-links store (doublets-rs + doublets-web) — FOUNDATION/BLOCKER

**Problem.** The durable store today is a custom `MemoryStore` backed by `.lino`
(`src/memory.rs`, `src/memory/bundle.rs`); the in-process event log
(`src/event_log.rs`) is rebuilt per request. `ARCHITECTURE.md` §4.2/§16.3 commits
to a doublet-links backend (`link-foundation/doublets-rs` on native,
`doublets-web`/IndexedDB in the browser) but the crate is not yet a dependency.
VISION.md says "doublet links are the primitive storage model"; until the store
*is* doublets, the network-is-the-AI invariants cannot be enforced uniformly
across CLI, HTTP, WASM, and Telegram.

**Approach.**
1. Define a `LinkStore` trait that the solver, memory, and event log all consume,
   with the existing `.lino` `MemoryStore` as one implementation (no behavior
   regression).
2. Add a `doublets-rs`-backed implementation on native targets and a
   `doublets-web` (IndexedDB) mirror behind the WASM target, selected at build
   time.
3. Make every persisted record reducible to doublets (`Type -> SubType -> Value`)
   with a stable, content-addressed id and a declared schema version. Reject
   ill-formed Links Notation on import.
4. Keep `.lino` export as the human-reviewable projection (GOALS.md: "binary
   stores … paired with reviewable Links Notation exports").

**Existing components.** `link-foundation/doublets-rs`, `doublets-web`;
`lino-objects-codec`, `lino-arguments` already in the repo; current `MemoryStore`
as the reference behavior to preserve.

**Acceptance criteria — graduate in `links_network.rs`:**
- `knowledge_export_is_reducible_to_doublet_links`
- `concepts_are_unique_and_referenced_by_id`
- `history_is_append_only`
- `knowledge_dataset_declares_schema_version`
- `records_are_addressable_by_stable_id`
- `ill_formed_links_notation_input_is_rejected`

Closes `ARCHITECTURE.md` §16.3. Blocks: E2, E5, E6, E9, E10, E13.

---

## E2 — Make the universal reasoning loop the only entry path — FOUNDATION/BLOCKER

**Problem.** The 11-step loop is documented and logged in `src/solver.rs`, but the
inner routing is a 35+ entry keyword/intent dispatch table
(`SPECIALIZED_HANDLERS`, `src/solver.rs:412`) plus 3 special-cased handlers. The
vision's "Universal Problem-Solving Algorithm" requires that *every* prompt walk
one formalize → search → decompose → candidates → validate → select pass, so the
trace is comparable across requests (NON-GOALS.md: "every prompt walks the same
loop").

**Approach.**
1. Re-seat the specialized handlers as **candidate generators** inside step 7
   (synthesis) rather than as the routing decision: the loop always records the
   impulse (step 1), formalizes (step 2), searches local then external
   (steps 3–4 via E1/E5), decomposes (step 5), generates candidates from handlers
   + reasoning (step 6–7), validates (step 8), selects the smallest sufficient
   answer (step 9), and returns it with a trace pointer.
2. Guarantee each step emits its event before the answer is projected; the answer
   is a projection of the log, never the source of record.
3. Preserve determinism and the green regression floor: existing handler outputs
   must still win for the prompts they already answer, now as scored candidates.

**Existing components.** `src/solver.rs` loop skeleton, `src/event_log.rs`,
`SolverConfig`, the existing handler functions (reused as candidate generators).

**Acceptance criteria — graduate in `reasoning_loop.rs` (all 11):**
`step_1_prompt_is_recorded_as_impulse`,
`step_2_local_search_runs_before_external_calls`,
`step_3_external_search_kicks_in_when_local_is_insufficient`,
`step_4_complex_requests_get_decomposed`,
`step_5_multiple_candidates_are_generated`,
`step_6_candidates_are_validated_against_constraints`,
`step_7_smallest_sufficient_answer_is_selected`,
`step_8_full_trace_is_stored_and_linked`,
`step_9_reply_is_returned_with_trace_pointer`,
`loop_terminates_on_unsolvable_questions`,
`confidence_reflects_corroborating_evidence`.

**And in `chat_surface.rs`:**
- `user_messages_are_recorded_as_impulse_events`
- `every_answer_exposes_a_trace_link_for_inspection`
- `unknown_intent_offers_a_path_to_extend_the_network`

Depends on: E1. Blocks: E3, E4, E6, E7, E8, E9.

---

## E3 — Full Wikidata P/Q-id formalization engine — FOUNDATION

**Problem.** Formalization is alias-based (`src/concepts.rs`). The vision needs
arbitrary prompts mapped to language-independent meanings anchored on Wikidata
**P-ids** (verbs/properties) and **Q-ids** (nouns/items), with
Wiktionary/Wikipedia as per-language fallbacks (`ARCHITECTURE.md` §5, §16.1).
This is the meaning layer every translation and reasoning step depends on.

**Approach.**
1. Build a multilingual labels table (cached via E5) and per-language morphology
   hints to resolve surface tokens to candidate Q/P-ids.
2. Emit formalization candidates as scored meaning records (consumed by E4's
   selection) with source links to the anchor.
3. Fall back to Wiktionary/Wikipedia surfaces when no Q/P-id exists; flag terms
   with no anchor as untranslatable inputs (feeds E6).

**Existing components.** Wikidata SPARQL (`src/translation/wikidata.rs`),
Wiktionary parser (`src/translation/wiktionary.rs`), `src/concepts.rs` aliases as
the seed; Abstract Wikipedia/Wikifunctions as prior art (`online-research.md`).

**Acceptance criteria.** Closes `ARCHITECTURE.md` §16.1. Add new tests covering
P/Q-id extraction over arbitrary prompts (verb→P-id, noun→Q-id, fallback to
surface, unknown→flagged). Enables the meaning-id tests in E6 and the
type-chain test in E10. Depends on: E2.

---

## E4 — Temperature-based interpretation selection + clarify-vs-guess

**Problem.** `SolverConfig.temperature` and `guess_probability`/`questioning_rigor`
exist, but there is no softmax/ε-comparison helper that turns candidate
formalization scores into a selection, and no clarify-vs-guess decision
(`ARCHITECTURE.md` §6, §16.2). The vision wants the system to ask **as few
questions as possible** (Q11) — guess when confident, ask the smallest question
when not.

**Approach.**
1. Add a deterministic softmax over candidate formalization scores (seeded from
   the impulse hash), with an ε-comparison to decide when the top candidate is
   clearly best.
2. When the gap is below threshold and `questioning_rigor` demands it, emit the
   smallest clarifying question instead of guessing; otherwise guess and record
   the chosen interpretation as a `candidate:` + `policy:` event.

**Existing components.** `SolverConfig` knobs, E3 candidate scores, `src/solver.rs`
formalization step.

**Acceptance criteria.** Closes `ARCHITECTURE.md` §16.2. Add tests: deterministic
selection for a fixed config; clarify path triggered under high rigor / low
margin; guess path under low rigor; same prompt+config ⇒ same choice. Contributes
to `confidence_reflects_corroborating_evidence` (owned by E2). Depends on: E2, E3.

---

## E5 — Public-knowledge source cache with provenance

**Problem.** External lookups must be cached with provenance and a refresh policy,
not treated as untracked context (GOALS.md; `ARCHITECTURE.md` §4.3). Offline mode
must refuse external lookups.

**Approach.**
1. Wrap external fetches (Wikidata/Wikipedia/Wiktionary/web) in a cache keyed by
   request, storing source URL, `fetched_at`, content hash, and TTL.
2. Record a `cache_hit:` event linking back to the prior `source:` record on
   reuse; refresh when stale; surface conflicting sources rather than silently
   picking one; honor an explicit, auditable flush; refuse lookups when
   `offline`.

**Existing components.** `src/translation/cache.rs`, `src/web_search_core.rs`,
`src/github_logs.rs`, the durable store from E1.

**Acceptance criteria — graduate in `source_cache.rs` (all 8):**
`external_lookups_record_source_url`,
`source_links_carry_fetched_at_timestamp`,
`stale_sources_are_refreshed`,
`repeated_lookups_hit_the_cache`,
`cached_sources_include_content_hash`,
`conflicting_sources_are_surfaced`,
`cache_flush_is_explicit_and_auditable`,
`offline_mode_disables_external_lookups`.

Depends on: E1.

---

## E6 — Translation via link-native meanings

**Problem.** The translation pipeline runs `formalize → meaning → deformalize`
(`src/translation/`), but the link-native invariants are not enforced: synonyms
across languages must share one meaning id, the trace must include the
intermediate meaning record, language tags must be declared, and untranslatable
concepts must be flagged.

**Approach.** Anchor meanings on the E3 Q/P-ids; persist one meaning id per
concept in the E1 store; render the target surface from per-language
labels/lexemes (Abstract Wikipedia/Wikifunctions model); attach the intermediate
meaning to the trace; flag terms with no anchor.

**Existing components.** `src/translation/pipeline.rs`, `wiktionary.rs`,
`wikidata.rs`; E3 formalization; E1 store.

**Acceptance criteria — graduate in `translation_via_links.rs` (all 7):**
`translation_preserves_meaning_id_across_languages`,
`translation_request_returns_target_surface_form`,
`synonyms_across_languages_share_meaning`,
`translation_declares_source_and_target_language_tags`,
`translation_trace_includes_intermediate_meaning`,
`cross_language_code_translation_preserves_semantics`,
`untranslatable_concepts_are_flagged`.

Depends on: E2, E3.

---

## E7 — Code generation & cross-language translation

**Problem.** Code answers must cover the top-10 popular languages, declare their
isolation level, include execution links, ship an algorithm **with tests** (TDD),
translate programs between languages while preserving semantics, and report
execution failures with a full trace.

**Approach.** Treat code generation as a loop sub-task: formalize the requested
algorithm/language, generate code + at least one test (step 6 TDD), attempt
execution where the surface allows it (link to E11 isolation), and emit
`trace:execution_failure` on failure instead of silently passing.

**Existing components.** `src/solver_helpers.rs`
(`build_sorting_algorithm_answer`, hello-world seeds), `src/solver_handlers/`
software-project handlers, the seed hello-world data.

**Acceptance criteria — graduate in `code_generation.rs` (all 6):**
`top_ten_popular_languages_each_have_a_hello_world_seed`,
`code_answers_include_execution_links_in_notation`,
`code_answers_declare_isolation_level`,
`sorting_algorithm_request_returns_code_and_tests`,
`translating_a_program_between_languages_keeps_semantics`,
`execution_failures_are_reported_with_full_trace`.

Depends on: E2; isolation level coordinates with E11.

---

## E8 — Formal reasoning engine (relative-meta-logic / SMT)

**Problem.** `src/proof_engine/` is a fixed classical-theorem registry. Issue Q9
asks for "formal reasoning that covers all current test cases **and much more**" —
a real decision procedure, not a hand-written theorem table.

**Approach.** Integrate `link-assistant/relative-meta-logic` and/or an SMT backend
(e.g. Z3) as a delegated, verified engine the loop can call for arithmetic,
constraints, and proofs — modeled on how `link-calculator` is already delegated.
Keep the proof presentation layer; replace the fixed table behind it with the
decision procedure, surfacing the proof/trace.

**Existing components.** `src/proof_engine/` (presenter to keep),
`link-assistant/calculator` (delegation model), `relative-meta-logic`; Lean/Z3
prior art (`online-research.md`).

**Acceptance criteria.** Closes issue Q9 / `ARCHITECTURE.md` §17 reasoning point.
Add tests proving theorems beyond the current fixed registry and arithmetic/
constraint checks via the decision procedure, while keeping the existing
`calculator_delegation.rs` and proof tests green. Depends on: E2.

---

## E9 — Chat-over-experience queries

**Problem.** The recorded experience must be queryable from chat: snapshot the
network, ask what is known about a concept, answer "why did you answer that?",
list "my facts", export the network as Links Notation, and require an explicit
retraction protocol for "forget X" — without leaking diagnostic ids into default
prose.

**Approach.** Add chat intents that project the E1 store/event log: network
snapshot, concept links, "why" (replay the prior trace), per-user fact filter,
`.lino` export, and a retraction event for "forget". Keep diagnostics off by
default, opt-in per message.

**Existing components.** `src/event_log.rs`, the E1 store, `src/solver_handlers/`
network-query handler, `transparent_state` spec as the contract.

**Acceptance criteria — graduate in `transparent_state.rs` (all 8):**
`querying_the_network_returns_snapshot`,
`querying_a_concept_returns_its_links`,
`diagnostic_ids_never_leak_into_default_chat_prose`,
`diagnostic_mode_can_be_enabled_per_message`,
`why_meta_question_explains_previous_answer`,
`forget_request_requires_explicit_retraction_protocol`,
`export_network_returns_links_notation_snapshot`,
`list_my_facts_filters_by_user`.

Depends on: E1, E2.

---

## E10 — Links-network invariants & dynamic type system

**Problem.** Beyond storage (E1), the network must publish the dynamic type system
as doublet subtype chains, attach a source link to every fact, attach a trace
link pointer to every answer, and list ordered reasoning steps in each trace
record (VISION.md type system; `links_network` spec remainder).

**Approach.** Implement the `Type -> SubType -> Value` chain projection over the
E1 store; enforce that every fact carries a `source:` link and every answer a
`trace:` pointer whose record lists ordered steps. These are cross-cutting
invariants checked on write.

**Existing components.** E1 store, E3 meaning records, `src/event_log.rs` trace
records; OpenCog AtomSpace prior art for type-as-data (`online-research.md`).

**Acceptance criteria — graduate in `links_network.rs` (the remaining 4):**
`dynamic_type_system_publishes_subtype_chains`,
`every_fact_carries_a_source_link`,
`every_answer_has_a_trace_link_pointer`,
`trace_record_lists_ordered_reasoning_steps`.

Depends on: E1, E2, E3.

---

## E11 — Agent mode with isolated execution

**Problem.** Agent mode is guarded but never executed: no sandbox, action log,
confirmation flow, time budget, secret guard, or privilege revocation. NON-GOALS
forbids unsafe agent use without isolation; chat must refuse unbounded multi-step
work without explicit opt-in.

**Approach.** Implement agent mode as an opt-in, explicitly-logged mode that runs
actions in an isolated environment (Docker / server sandbox / browser VM where
practical), appends every action to a visible log, surfaces failures, requires
confirmation for destructive actions, enforces a time budget, never leaks host
env vars, and revokes privileges when switching back to chat.

**Existing components.** `SolverConfig.agent_mode`, `src/telegram_runtime.rs`
(Docker-in-Docker runtime), `agent_isolation` + `chat_surface` specs.

**Acceptance criteria — graduate in `agent_isolation.rs` (all 9):**
`agent_mode_is_off_by_default`,
`agent_mode_opt_in_is_explicit_and_logged`,
`agent_execution_runs_in_isolated_environment`,
`agent_actions_are_appended_to_visible_log`,
`agent_failures_are_visible`,
`destructive_agent_actions_require_confirmation`,
`agent_mode_enforces_time_budget`,
`agent_mode_does_not_leak_host_env_vars`,
`switching_to_chat_revokes_agent_privileges`.

**And in `chat_surface.rs`:**
- `chat_mode_refuses_unbounded_multi_step_actions_without_agent_opt_in`

Depends on: E2.

---

## E12 — Authenticated API + tool-call gating

**Problem.** The OpenAI-compatible API must accept a bearer token on authenticated
routes and must refuse tool calls unless agent mode is on (`openai_compatibility`
spec; ties to E11's isolation contract).

**Approach.** Add bearer-token auth to the HTTP routes and gate
`tool_calls`/function execution behind `agent_mode`, returning a clear refusal
otherwise.

**Existing components.** `src/protocol.rs`, the HTTP server in `src/main.rs`,
`SolverConfig.agent_mode`.

**Acceptance criteria — graduate in `openai_compatibility.rs` (both):**
`authenticated_routes_accept_bearer_token`,
`chat_completion_refuses_tool_call_without_agent_mode`.

Depends on: E11.

---

## E13 — Network visualization + trace links on every surface

**Problem.** The link graph must be available beside chat without ever blocking
replies (web demo), Telegram answers must carry a tap-to-inspect trace link, and
every code answer must declare its execution status while diagnostics stay out of
default prose.

**Approach.** Surface the E1/E10 trace pointers on every surface: a
non-blocking graph panel in the web demo, a trace link in Telegram replies, an
explicit execution-status line on code answers, and diagnostics gated behind the
opt-in flag.

**Existing components.** `src/web/` demo, `src/telegram.rs`,
`network_visualization` + `telegram_surface` + `chat_surface` specs.

**Acceptance criteria:**
- `network_visualization.rs`: `web_demo_chat_works_even_when_graph_is_disabled`
- `telegram_surface.rs`: `telegram_answers_include_trace_link`
- `chat_surface.rs`: `every_code_answer_declares_execution_status_or_unavailability`,
  `diagnostics_are_excluded_from_default_user_facing_answers`

Depends on: E2, E10.

---

## E14 — Natural-language skill compilation

**Problem.** VISION.md's computation model has five rule shapes ending in
**natural-language skills**; today every skill is interpreted by the universal
solver step by step, with no compiler (`ARCHITECTURE.md` §9 #5, §16.4).

**Approach.** Add a compiler that turns a natural-language skill description into a
reusable associative package (substitution rules / triggers / compiled handler)
in the E1 store, so a learned procedure can be replayed without re-deriving it —
while keeping every compiled skill reviewable as Links Notation and traceable.

**Existing components.** `src/seed/` (rule/skill seeds), `src/solver_handlers/`
(handlers as the compiled form), E1 store; OpenCog MeTTa prior art for
self-modifying rules (`online-research.md`).

**Acceptance criteria.** Closes `ARCHITECTURE.md` §16.4. Add tests: a
natural-language skill compiles to a reusable package; the compiled package
replays deterministically and is exportable as Links Notation; a compiled skill
is preferred (recorded as `cache_hit:`) over re-deriving. Depends on: E2, E10.

---

## E15 — Make doublets-rs the default native physical store — [#278](https://github.com/link-assistant/formal-ai/issues/278)

**Problem.** Issue #246 introduced the `LinkStore` boundary and doublet
projections, but the native durable store is still the reviewable `.lino`
projection unless a feature-backed implementation is selected.
`ARCHITECTURE.md` still lists the native physical-store migration as a remaining
question, and `src/link_store.rs` explicitly keeps the `.lino` implementation as
the current default.

The vision says the associative network is the AI. That requires the native
runtime to use `doublets-rs` as the default physical store, while keeping Links
Notation as the auditable export/import projection.

**Scope.**
- Make `doublets-rs` the default native physical backend for persisted link
  records.
- Preserve `.lino` export/import as the human-reviewable projection and migration
  format.
- Keep browser storage compatible with `doublets-web`/IndexedDB expectations.
- Add migration coverage from existing `.lino` bundles into the default native
  store.
- Document how to recover, inspect, and export the store.

**Acceptance criteria.**
- A default native build persists link records through `doublets-rs` without
  requiring an opt-in feature for the primary store path.
- Existing `.lino` memory bundles import into the native store and export back to
  deterministic Links Notation.
- CLI, HTTP, library, and Telegram surfaces continue to share the same store
  semantics.
- Tests cover migration, stable IDs, append-only history, malformed import
  rejection, and feature/fallback behavior.
- `ARCHITECTURE.md`, `ROADMAP.md`, and `REQUIREMENTS.md` no longer describe the
  native doublets store as future work.

**Requirement links.** `REQUIREMENTS.md` R60, `ARCHITECTURE.md` section 16,
follow-up from #246.

---

## E16 — Symbolic probabilistic reasoning over Links Notation — [#279](https://github.com/link-assistant/formal-ai/issues/279)

**Problem.** `REQUIREMENTS.md` R6 asks us to explore Bayesian networks, Markov
chains, and similar symbolic/probabilistic methods. The current implementation
has deterministic rules and the new temperature-based selection path, but it
does not yet store or update probabilistic evidence over the Links Notation
graph.

We need a symbolic probabilistic layer that helps rank interpretations and
candidate answers without using neural-network inference for reasoning.

**Scope.**
- Represent probabilistic evidence as link-native records, with provenance and
  timestamps.
- Add Bayesian/Markov-style ranking helpers over candidate formalizations or
  answer candidates.
- Integrate ranking with the existing temperature/clarify-vs-guess selection
  policy.
- Keep deterministic replay: same prompt, same store, same config, and same
  impulse hash must produce the same selected candidate.
- Surface probability evidence in traces so users can inspect why a candidate
  outranked another.

**Acceptance criteria.**
- Tests demonstrate link-native probabilistic evidence creation, update, and
  replay.
- Candidate ranking changes when new symbolic evidence is added, without
  modifying neural weights or calling neural inference.
- The clarify-vs-guess policy can consume the probability margin between top
  candidates.
- Offline mode and cached-source provenance remain respected.
- Documentation explains the supported probabilistic model and its non-neural
  boundary.

**Requirement links.** `REQUIREMENTS.md` R6, `VISION.md` universal
problem-solving and no-neural-reasoning constraints.

---

## E17 — Desktop application wrapper for formal-ai — [#280](https://github.com/link-assistant/formal-ai/issues/280)

**Problem.** `REQUIREMENTS.md` R17 calls for a desktop application path similar
to `vk-bot-desktop`. The repository now has CLI, HTTP, library, Telegram, and
browser/WASM surfaces, but no packaged desktop shell that reuses those surfaces
for local users.

**Scope.**
- Package a desktop application wrapper around the existing web/chat/network
  surfaces.
- Reuse the library/HTTP API instead of forking solver behavior.
- Support local memory bundle import/export and inspectable trace/network views.
- Document development, packaging, and release steps for the desktop app.
- Keep secrets, agent mode, and tool-call permissions explicit in the desktop UI.

**Acceptance criteria.**
- A developer can run the desktop app locally from documented commands.
- The desktop app can send prompts, show answers, inspect traces/network links,
  and import/export memory bundles.
- Desktop behavior is covered by at least one smoke/integration test or
  documented manual verification script.
- Agent mode and tool-call actions remain opt-in and permission-gated.
- `REQUIREMENTS.md` R17 is updated from future work to implemented or explicitly
  scoped to any remaining sub-issues.

**Requirement links.** `REQUIREMENTS.md` R17, related prior art:
`vk-bot-desktop`.

---

## E18 — Reusable associative packages and permission model — [#281](https://github.com/link-assistant/formal-ai/issues/281)

**Problem.** Issue #259 added deterministic natural-language skill compilation,
and issue #257 added API authentication/tool-call gating. `REQUIREMENTS.md` R65
still points to a broader Deep.Foundation-style model: associative packages,
handlers, permissions, and trigger-style computation represented as reusable
link-native data.

The current implementation has pieces of that model, but not a complete
package/permission system that can install, export, import, replay, and gate
reusable associative packages.

**Scope.**
- Define package metadata, dependency links, exported handlers, permissions, and
  trigger records in Links Notation.
- Allow compiled skills and handler registrations to belong to packages.
- Require explicit permissions for package-provided tools or actions.
- Support deterministic install/export/import/replay of packages.
- Surface package and permission records in traces and network visualization.

**Acceptance criteria.**
- Tests cover package definition, dependency validation, install, export/import,
  replay, and permission denial.
- A compiled skill can be packaged and imported without hand-editing Rust code.
- Tool-call gating can identify whether a package grants the required
  capability.
- The network view exposes package, handler, trigger, and permission links.
- Documentation maps this design back to R65 and the Deep.Foundation-inspired
  requirement.

**Requirement links.** `REQUIREMENTS.md` R65, follow-up from #257 and #259.

---

## E19 — Complete Rust-to-WebAssembly solver parity for the browser worker — [#282](https://github.com/link-assistant/formal-ai/issues/282)

**Problem.** `REQUIREMENTS.md` R194 asks for as much logic as possible to be
compiled from Rust to WebAssembly, with JavaScript reserved for UI. The browser
worker now delegates several core operations to Rust/WASM through
`web_engine_core`, but significant browser-worker behavior remains in
JavaScript.

We need a parity effort that moves remaining solver/domain logic into Rust/WASM
while keeping JavaScript as UI, fetch, and integration glue.

**Scope.**
- Inventory browser-worker logic and classify it as domain logic vs UI/fetch
  glue.
- Move remaining reusable solver/domain logic into Rust/WASM modules.
- Keep JavaScript fallbacks only where needed for compatibility, with explicit
  tests.
- Add parity tests that compare native Rust and browser/WASM behavior for
  representative prompts and memory operations.
- Update documentation with the Rust/WASM boundary.

**Acceptance criteria.**
- Browser answers for selected prompts match native Rust behavior for normalized
  output, traces, and evidence records.
- JavaScript worker code no longer owns reusable domain decisions that can live
  in Rust.
- WASM build and browser tests cover the moved behavior.
- Documentation describes the remaining JavaScript responsibilities as
  UI/fetch/integration glue.
- `REQUIREMENTS.md` R194 reflects the completed boundary or lists any
  intentionally retained JS exceptions.

**Requirement links.** `REQUIREMENTS.md` R194, follow-up from the browser/WASM
work in issues #246-#259.

---

## E20 — Generalized natural-language skill compiler beyond trigger/response — [#283](https://github.com/link-assistant/formal-ai/issues/283)

**Problem.** Issue #259 implemented a deterministic natural-language skill
compiler for trigger/response rules. `ARCHITECTURE.md` still calls out the next
step: broader lowering of natural-language skill definitions into executable
Rust/JavaScript/native handlers and package data.

The current compiler is useful, but it does not yet cover typed arguments,
multi-step procedures, validations, generated tests, or target-specific handler
lowering.

**Scope.**
- Extend the skill language beyond trigger/response into typed inputs,
  preconditions, steps, effects, and expected tests.
- Lower compatible skills into package records and, where appropriate, generated
  Rust/JavaScript/native handler stubs.
- Keep generated behavior deterministic and traceable.
- Refuse or mark unsupported natural-language instructions instead of silently
  compiling unsafe behavior.
- Integrate with package/permission records from the associative-package
  follow-up.

**Acceptance criteria.**
- Tests cover typed skill definitions, multi-step procedures, generated tests,
  unsupported-instruction refusal, and deterministic replay.
- Generated handlers or package records can be inspected as Links Notation.
- Unsafe or permissioned actions require explicit package/tool permissions.
- Documentation distinguishes the supported skill subset from future
  natural-language programming goals.
- `ARCHITECTURE.md` no longer lists skill compilation as only trigger/response
  work.

**Requirement links.** `ARCHITECTURE.md` section 16, `REQUIREMENTS.md` R65, and
the issue #244 skill/learning requirements; follow-up from #259.

---

## Coverage check

In the original 2026-05-25 plan, every one of the 69 `#[ignore]`
tracked-requirement tests was assigned to exactly one epic:

| Spec file | Tests | Epic(s) |
| --- | --- | --- |
| `reasoning_loop.rs` | 11 | E2 |
| `links_network.rs` | 10 | E1 (6) + E10 (4) |
| `agent_isolation.rs` | 9 | E11 |
| `transparent_state.rs` | 8 | E9 |
| `source_cache.rs` | 8 | E5 |
| `translation_via_links.rs` | 7 | E6 |
| `code_generation.rs` | 6 | E7 |
| `chat_surface.rs` | 6 | E2 (3) + E11 (1) + E13 (2) |
| `openai_compatibility.rs` | 2 | E12 |
| `telegram_surface.rs` | 1 | E13 |
| `network_visualization.rs` | 1 | E13 |
| **Total** | **69** | |

The 2026-05-26 audit confirmed that no tracked ignored tests remain under
`tests/unit/specification/`. The E15-E20 batch then closed the smaller follow-up
requirements. The remaining gap — reasoning behaviour rather than storage,
surfaces, or compilation — is now tracked by the E21-E27 batch below.

---

## Reasoning Batch (E21-E27)

The third-pass 2026-05-26 audit, acting on issue #244 feedback, found that the
solver still routes on a fixed intent catalogue and falls back to a canned opener
on unmatched prompts instead of reasoning under unknowns. These seven epics move
the assistant from intent-matching to reasoning. They are ordered
foundation-first: E22 (intent formalization) and E21 (reasoning under unknowns)
underpin E23-E26, and E27 supplies the unseen-task corpus E26 is measured on.
Each epic names the exact code symbol it must replace or extend.

### E21 — Reasoning under unknowns instead of failing — [#298](https://github.com/link-assistant/formal-ai/issues/298)

**Problem.** On an unmatched prompt the solver emits a canned "I can't answer
that" opener. `language_aware_unknown_answer()` in `src/unknown_opener.rs` only
swaps the opener of a fixed teaching message by a stable hash; it performs no
reasoning. `handle_specialized_pattern()` (`src/solver.rs`) gives up when no
`SPECIALIZED_HANDLERS` entry matches. This contradicts VISION.md ("work with
unknowns by reasoning and gathering the missing data").

**Scope.** On an unmatched prompt, run real reasoning steps before giving up:
identify knowns, identify what is missing, choose an accessible source (link
memory, public-knowledge cache, allowed API), attempt to gather it, and record
each step as links. Ask at most one minimal clarifying question only after
exhausting reachable options.

**Existing components to reuse.** `src/event_log.rs` reasoning-step event kinds;
`src/solver.rs` universal-loop scaffolding and `SolverConfig`
(`questioning_rigor`, `offline`, `cache_ttl_seconds`); the E5 source cache.

**Acceptance criteria.** Reasoning-step trace emitted (not a single sentence);
reachable missing facts retrieved and answered; minimal-question path; canned
fallback used only as a recorded last resort; new specification tests for each
path.

**Requirement links.** Issue #244 reasoning-under-unknowns feedback; VISION.md
universal problem-solving loop; `src/unknown_opener.rs`.

### E22 — Intent formalization as Links Notation — [#299](https://github.com/link-assistant/formal-ai/issues/299)

**Problem.** Routing is driven by a fixed catalogue: `select_rule_for()` maps
prompts onto the closed `SelectedRule` enum (`src/engine.rs`) and
`SPECIALIZED_HANDLERS` (`src/solver.rs`) is an ordered keyword table. The
Wikidata-backed `FormalizationCandidate` (`src/translation/formalization.rs`) is
not the primary router. VISION.md asks for intent as Links Notation / binary
links, with prior reasoning cached.

**Scope.** Formalize every message into a `.lino` intent (kind, knowns,
relevants) before routing; route from the formalized intent; cache prior
formalizations keyed by content-addressed impulse id so repeated messages are not
re-reasoned.

**Existing components to reuse.** `src/translation/formalization.rs`;
`src/concepts.rs` alias fallback; `src/memory.rs` `.lino` store; Links Notation
tooling (`lino-objects-codec`, `lino-arguments`).

**Acceptance criteria.** Prompt → intent links recorded; routing derived from the
intent links; cache hit on repeated messages; `FormalizationCandidate` wired into
routing or explicitly superseded; new specification tests.

**Requirement links.** Issue #244 "formalize the messages of the user" feedback;
ARCH §5 formalization.

### E23 — Generalized parametric intents — [#300](https://github.com/link-assistant/formal-ai/issues/300)

**Problem.** Code generation is a per-language enumeration: `HELLO_WORLD_PROGRAMS`
(`src/engine_hello_world.rs`) lists ~10 languages, each effectively its own
intent. Issue #244: "we should have `write a program` intent with parameters."
Depends on E22.

**Scope.** Collapse per-language intents into one parametric
`write a program(language, task)` intent resolved from the formalized intent;
remove or data-drive `HELLO_WORLD_PROGRAMS`; handle an unlisted supported language
through the same path without new routing code.

**Existing components to reuse.** E22 formalized intent; E7 code generation
(`code_generation.rs`); `src/engine_hello_world.rs` templates as seed data.

**Acceptance criteria.** One parametric intent serves all current hello-world
languages from parameters; no per-language routing branch; unlisted supported
combination handled; existing tests pass; graceful rejection of unsupported
parameters.

**Requirement links.** Issue #244 parametric-intent feedback; E7.

### E24 — Substitution-rule handlers over link CRUD — [#301](https://github.com/link-assistant/formal-ai/issues/301)

**Problem.** Behaviour is only Rust functions plus the narrow
`When I say "X", answer "Y"` shape in `src/skill_compiler.rs`. There is no general
`replace x y` over patterns and no composition into `when n do m`. Issue #244 asks
for `link-cli`-style substitution operations as data on link CRUD.

**Scope.** Add a `replace x y` pattern-substitution primitive over link patterns
(with variable binding), composable into conditional `when … do …` rules,
attachable to link CRUD events, deterministic, and inspectable as Links Notation —
alongside the existing Rust handlers.

**Existing components to reuse.** `link-foundation/link-cli` semantics;
`src/skill_compiler.rs` / `src/associative_package.rs` for rule-as-data lowering;
`src/memory.rs` CRUD path; `src/event_log.rs` trace links.

**Acceptance criteria.** Single `replace`; composed `when n do m`; CRUD-attached
rule firing; determinism/termination guard; a behaviour previously needing a Rust
handler expressed purely as rule data; new specification tests.

**Requirement links.** Issue #244 substitution-rule feedback; ARCH §16; OpenCog
AtomSpace/MeTTa prior art (`raw-data/online-research.md` §3).

### E25 — Natural-language access to memory, APIs, and code execution — [#302](https://github.com/link-assistant/formal-ai/issues/302)

**Problem.** NL cannot yet drive the three core capabilities generally: memory
queries are not answerable from arbitrary prompts; only a few specialized API
handlers exist (`http_fetch`, `web_search` in `SPECIALIZED_HANDLERS`); there is no
runtime code execution in the core. Depends on E22 and the E12/E18 permission
model.

**Scope.** From the formalized intent, query the link memory, select and call an
allowed API, and execute code — all gated by the permission model and recorded as
trace links with declared execution status.

**Existing components to reuse.** `SPECIALIZED_HANDLERS` http/web/exec stubs; E12
auth + tool-call gating; E18 package permissions; `transparent_state.rs`,
`chat_surface.rs`, `openai_compatibility.rs` specs.

**Acceptance criteria.** NL memory queries answered from the link store; NL API
call gated and recorded; NL code execution only when permitted; execution status
declared; permission-denied path tested.

**Requirement links.** Issue #244 "use natural language to query memory / access
APIs / execute code" feedback; E12/E18.

### E26 — General code-modifying / executing agent — [#303](https://github.com/link-assistant/formal-ai/issues/303)

**Problem.** The system memorizes code (`HELLO_WORLD_PROGRAMS`) and agent mode is
gated but never executes (no sandbox, action log, terminal actions). Issue #244:
"our goal is to have an AI system that actually can code anything … write, modify,
execute, and run terminal actions … proven by a much larger automated test
suite." Depends on E23, E24, E25, and E11 isolation.

**Scope.** A general coding agent that writes, modifies, executes, and runs
terminal actions in the E11 isolated workspace with action log, confirmation,
time budget, secret guard, and revocation; correctness proven by a substantially
larger test suite (including unseen E27 tasks), not by seed matching.

**Existing components to reuse.** E11 agent isolation (`agent_isolation.rs`); E25
execution path; E23 parametric intent; E7 code generation; E27 benchmarks.

**Acceptance criteria.** Isolated file create/modify/delete + terminal commands
with action log; unseen coding tasks solved by iterate-not-lookup; sandbox /
confirmation / time budget / secret guard / revocation enforced and tested; test
suite grows substantially; regression floor intact; execution status surfaced.

**Requirement links.** Issue #244 general-coding-agent feedback; E11; NON-GOALS.md
isolation requirement.

### E27 — Industry benchmark datasets — [#304](https://github.com/link-assistant/formal-ai/issues/304)

**Problem.** The corpus is own seeds + specification tests only; no industry
benchmarks. Without them, "can code anything / solve problems" is only measured
against memorizable prompts. Issue #244: "double check industry leading datasets …
available in permissible licenses … import as test cases."

**Scope.** Import permissively-licensed programming, problem-solving, and math
benchmarks as deterministic `.lino` test cases with recorded license provenance;
wire them into the harness as a benchmark suite that forces generalization over
seed memorization.

**Candidate datasets (verify license before import).** HumanEval (MIT), MBPP
(CC-BY-4.0/Apache-2.0), GSM8K (MIT), MATH (MIT). Document the exact license and
source for whatever is imported; reject non-permissive or unclear sources.

**Existing components to reuse.** `tests/unit/specification/` harness and `.lino`
fixtures; `raw-data/online-research.md` for the license/provenance survey.

**Acceptance criteria.** At least one programming, one math, and one
problem-solving benchmark imported with recorded permissive license + provenance;
each represented as deterministic `.lino` test cases in a runnable benchmark
suite reporting pass/fail; a research note listing each dataset, license, size,
and selection rationale; no verbatim copying beyond licensed dataset content.

**Requirement links.** Issue #244 industry-dataset feedback; program-synthesis /
theorem-proving evaluation framing (`raw-data/online-research.md` §4).

## Reasoning-batch coverage check

Each E21-E27 epic maps to exactly one of the six third-pass audit findings (plus
E27 as the measurement corpus):

| Audit finding (code anchor) | Epic |
| --- | --- |
| Canned unknown fallback (`src/unknown_opener.rs`) | E21 |
| Fixed intent catalogue (`SelectedRule`, `SPECIALIZED_HANDLERS`) | E22 |
| Per-language enumeration (`HELLO_WORLD_PROGRAMS`) | E23 |
| Trigger/response only (`src/skill_compiler.rs`) | E24 |
| No NL memory/API/exec path; no runtime execution | E25 |
| Memorized code; agent gated-but-unexecuted | E26 |
| Own seeds only; no industry benchmarks | E27 |

---

## Synthesis Batch (E28-E32) — CLOSED (PRs #319-#323)

The fourth-pass 2026-05-27 audit, acting on issue #244 feedback, found that the
universal 11-step loop is the main path for every prompt and the E21-E27 batch
made it formalize intents, reason under unknowns, run substitution rules, and
drive a bounded agent — but the synthesis step (`record_candidates`) still
resolved answers from seeded handlers rather than deriving them. The imported
industry benchmark suite made this concrete: it passed **0/5**. These five epics
made the synthesis step **derive** solutions over the links network, one
benchmark domain at a time, with the benchmark pass count as the objective metric
and an anti-memorization rule binding every epic (no answer string keyed on the
prompt; paraphrased/renumbered held-out variants must pass only via derivation).
They are ordered foundation-first: E28 is the general synthesis substrate that
E29-E31 build on, and E32 grows and ratchets the measurement corpus.

**Outcome (2026-05-29):** the batch is merged (PRs #319-#323). The synthesis step
now derives answers and the benchmark suite grew to a 10-case slice that passes
**10/10** with a `minimum_pass_count` ratchet.

The full bodies opened in the repository are reproduced below.

## E28 — General link-native synthesis substrate (derive candidates, don't seed them) — FOUNDATION/BLOCKER

Parent: #244. Part of the **synthesis batch (E28-E32)** opened by the 2026-05-27 fourth-pass audit.

**Problem.** The universal 11-step loop is the main path for every prompt (`src/solver.rs::solve_with_history_probability_store_and_intent_cache`), and the E21-E27 batch made it formalize intents, reason under unknowns, run substitution rules, and drive a bounded agent. But the synthesis step still *looks up* answers: `record_candidates` (`src/solver.rs`, `src/solver_helpers.rs`) and the `SPECIALIZED_HANDLERS` precedence table resolve a prompt to a seeded handler output rather than **deriving** a result by composing the decomposed sub-problems over the links network. The concrete symptom is that the imported industry benchmark suite (`tests/unit/specification/benchmarks.rs`, `data/benchmarks/industry-suite.lino`) passes **0/5**.

This contradicts VISION.md's "doublet links let us see any data and knowledge as links, so we can have truly universal algorithms that operate directly on links" — the synthesis step must compose sub-results over the links network, not return a memorized string keyed on the prompt.

**Scope.**
1. Make `record_decomposition` produce sub-impulses that the synthesis step actually consumes: each sub-impulse is solved (recursively, bounded by `max_decomposition_depth`) and recorded as a link.
2. Make `record_candidates` build candidates by **composing** the sub-results over the links network (a candidate is a composition of solved sub-links), so the answer is a projection of the recorded sub-solutions, never a seed lookup.
3. Keep the existing handlers as **candidate generators** inside synthesis (their outputs become scored candidates), not as the routing decision — preserving the green regression floor.
4. Record every composition step as a link so "why did you answer that?" replays the actual derivation.

**Existing components to reuse.** `src/solver.rs` loop (`record_decomposition`, `record_candidates`, `record_validation`), `src/solver_helpers.rs`, `src/event_log.rs`, the E22 formalized intent, the E16 probabilistic ranking for candidate selection.

**Acceptance criteria.**
- Decomposed sub-impulses are solved and recorded as links, and the final answer is built by composing those sub-result links (assert the trace contains the sub-result links the answer is composed from).
- At least one benchmark domain moves from failing toward passing purely through composition, **without** adding a per-case answer string keyed on the prompt (anti-memorization: a paraphrased prompt with the same structure must reach the same derivation).
- Determinism preserved: same prompt + same `SolverConfig` ⇒ same composition trace and answer.
- Regression floor intact: the currently-green spec files stay green; handlers now win as scored candidates.
- New specification tests in a `synthesis.rs` (or extended `reasoning_loop.rs`) covering decompose→solve-subparts→compose.

**Requirement links.** Issue #244 "general algorithm for everything, operating directly on links"; `ROADMAP.md` pillar 26. Blocks: E29, E30, E31.

---

## E29 — Compute math/word-problem and counting answers (don't seed them) — [GSM8K, MATH, BIG-bench]

Parent: #244. Part of the **synthesis batch (E28-E32)**.

**Problem.** Three of the five failing benchmark cases are numeric reasoning the solver should *compute*, not look up:
- `gsm8k_test_0_duck_eggs` expects `18` (a multi-step arithmetic word problem),
- `math_train_7_algebra_substitution` expects `11` (algebraic substitution),
- `bigbench_object_counting_instruments` expects `3` (count items of a category from a list).

Today `record_candidates` resolves these from seeded handlers, so the suite reports them as failing (`benchmark pass/fail counts: passed=0 failed=5`). VISION.md forbids neural inference for reasoning; the answer must come from a deterministic decision/arithmetic procedure over the formalized, decomposed problem.

**Scope.**
1. Word-problem arithmetic: decompose a GSM8K-style prompt into its quantities and operations (via E28 decomposition), evaluate them deterministically through `src/proof_engine/` / the calculator delegation path, and compose the numeric result.
2. Algebraic substitution: extend the decision procedure to substitute and evaluate the MATH-style expression to a closed value.
3. General counting/aggregation: count items of a requested category from an enumerated list (BIG-bench object_counting) as a general links-aggregation over the parsed entities, not a per-prompt seed.
4. Surface the computation in the trace (the steps, not just the number), and declare the result with a `trace:` pointer.

**Existing components to reuse.** `src/proof_engine/decision.rs` (boolean/linear decision modules), `calculator_delegation.rs` model, E28 decomposition/composition, `src/intent_formalization.rs` for extracting quantities and the target.

**Acceptance criteria.**
- `gsm8k_test_0_duck_eggs`, `math_train_7_algebra_substitution`, and `bigbench_object_counting_instruments` pass, with the answer computed from the decomposed problem (assert the trace shows the intermediate quantities/steps).
- Anti-memorization: changing the numbers in a structurally identical prompt yields the correctly recomputed answer (add at least one paraphrased/renumbered held-out case per domain).
- Deterministic and offline-safe; no neural inference used for the reasoning.
- The benchmark suite's reported pass count rises accordingly; regression floor intact.

**Requirement links.** Issue #244 "do everything as humans do, but algorithmically"; `ROADMAP.md` pillar 26; depends on E28.

---

## E30 — General program synthesis from spec + tests (derive and verify, don't memorize) — [HumanEval, MBPP]

Parent: #244. Part of the **synthesis batch (E28-E32)**.

**Problem.** Two failing benchmark cases require writing a program from a specification:
- `humaneval_0_has_close_elements` expects ```` ```python ```` and `def has_close_elements`,
- `mbpp_2_similar_elements` expects ```` ```python ````, `def similar_elements`, and `set`.

Today `SelectedRule::WriteProgram` resolves these from seed templates (`src/seed.rs` / `engine_hello_world.rs`), so they fail when the requested function is not a seeded hello-world. The bounded agent built in E26 (`src/agent.rs`) can run allowlisted commands in an isolated workspace, but the loop does not yet **synthesize** a novel function from the spec and **verify** it against the provided tests. Issue #312 ("Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории") is a concrete real-world instance of the same gap.

**Scope.**
1. Treat a `write a program` request as a synthesis sub-task: from the formalized intent + the provided/derived tests, derive a candidate function body (composing known link-native code fragments via E28), not a seed lookup.
2. Verify each candidate by executing it (and its tests) in the E26 bounded agent workspace; iterate on failures (TDD step) rather than returning the first guess.
3. Return the first candidate that passes the tests, with the execution status and the test outcome recorded in the trace.
4. Keep memorized seeds only as one candidate generator among others; a previously-seeded program must still be reproducible, but unseen specs must be solved by derive-and-verify.

**Existing components to reuse.** `SelectedRule::WriteProgram` (`src/solver.rs`), `src/agent.rs` bounded workspace, E25 permissioned execution path, E28 composition, `code_generation.rs` and `agent_isolation.rs` specs.

**Acceptance criteria.**
- `humaneval_0_has_close_elements` and `mbpp_2_similar_elements` pass by deriving the function and verifying it against tests in the isolated workspace (assert the trace records candidate execution + test pass, not a seed hit).
- Anti-memorization: at least one additional unseen HumanEval/MBPP-style function (not present as a seed) is solved by the same path.
- Issue #312's "list files" program is produced and (where the surface allows) executed, with declared execution status.
- Sandbox/time-budget/secret-guard from E26 enforced; deterministic where execution is deterministic; regression floor intact.

**Requirement links.** Issue #244 "write programs, algorithms as asked … change code"; issue #312; `ROADMAP.md` pillar 26; depends on E28, E26.

---

## E31 — General text manipulation over arbitrary user input (transform / extract / count / rewrite)

Parent: #244. Part of the **synthesis batch (E28-E32)**.

**Problem.** Issue #244 asks that we "do any text manipulation task on any text input the user will give us." Today text operations are handled by a scattered set of specialized handlers; there is no general path that treats an arbitrary "do X to this text" request as a composition of substitution rules over the links network. The E24 substitution engine (`src/substitution.rs`, `replace x y` / `when n do m`) exists but is not yet the general route for user-supplied text transforms.

**Scope.**
1. Route a "transform/extract/count/rewrite this text" request through the formalized intent into a composition of E24 substitution rules over the parsed text links (uppercase/lowercase, replace, reverse words, extract matches, count occurrences, deduplicate, sort lines, etc.).
2. Operate on the user-supplied input directly (collect only the minimum required data — the text and the requested operation — no memoized per-input answers).
3. Record the rule chain in the trace so the transformation is inspectable and replayable as Links Notation.
4. Compose multiple operations ("lowercase then count unique words") via E28 composition rather than a dedicated handler per combination.

**Existing components to reuse.** `src/substitution.rs` (E24), `substitution_rules.rs` spec, E22 formalized intent, E28 composition, `src/event_log.rs` trace links.

**Acceptance criteria.**
- A representative set of text-manipulation requests (transform, extract, count, rewrite, and at least one composed multi-step) is solved over arbitrary user-supplied input through composed substitution rules, with the rule chain recorded in the trace.
- Anti-memorization: the same operation on different input text produces the correctly recomputed result (no per-input seed).
- Deterministic; minimum-data (only the supplied text + operation are consumed); regression floor intact.
- New specification tests covering each operation class plus a composed case.

**Requirement links.** Issue #244 "any text manipulation task on any text input"; `ROADMAP.md` pillar 26; depends on E28, E24.

---

## E32 — Grow the benchmark suite and gate progress on rising pass counts (anti-memorization)

Parent: #244. Part of the **synthesis batch (E28-E32)**.

**Problem.** The E27 import (`data/benchmarks/industry-suite.lino`, `tests/unit/specification/benchmarks.rs`) is a deliberately small 5-case slice, and every case is `allow_current_failure` so the suite reports `passed=0 failed=5` without failing CI. That is the right starting contract, but it does not yet (a) measure generalization on held-out cases, or (b) ratchet: nothing prevents a future regression from silently dropping the pass count, and nothing forces E29-E31 progress to stick.

**Scope.**
1. Grow the suite beyond 5 cases per the upstream permissive licenses already documented in `data/benchmarks/LICENSES.md` (more HumanEval/MBPP/GSM8K/MATH/BIG-bench cases), keeping provenance and `source_ref` per source.
2. Add **held-out / paraphrased** variants (renumbered word problems, paraphrased specs, different input text) that share structure with seeded examples, so a memorized answer cannot pass them — the anti-memorization guard for E29-E31.
3. Add a monotonic **pass-count floor**: a recorded baseline pass count that CI asserts the suite meets or exceeds, so E29-E31 gains cannot silently regress, while `allow_current_failure` still keeps unmet cases from breaking CI.
4. Keep the report deterministic and offline-safe; surface the pass/fail breakdown for reviewers.

**Existing components to reuse.** `tests/unit/specification/benchmarks.rs`, `data/benchmarks/industry-suite.lino`, `data/benchmarks/LICENSES.md`, `docs/case-studies/issue-244/raw-data/online-research.md` provenance notes.

**Acceptance criteria.**
- The suite contains more than 5 cases with held-out/paraphrased variants, each with recorded permissive license + `source_ref`.
- CI asserts a recorded minimum pass count (ratchet) that rises as E29-E31 land, and fails if the pass count drops below the recorded floor.
- A held-out variant of each solved domain passes only via derivation (proving no per-case memorization).
- No verbatim copying beyond licensed dataset content; provenance note updated.

**Requirement links.** Issue #244 "solid verification using test coverage that we really can solve these"; `ROADMAP.md` pillar 26; depends on E29, E30, E31.

---

## Synthesis-batch coverage check

Each E28-E32 epic maps to a concrete failing benchmark case (or the substrate /
measurement they depend on):

| Failing benchmark case (expected) | Epic |
| --- | --- |
| (substrate for all five) `record_candidates` derives instead of seeds | E28 |
| `gsm8k_test_0_duck_eggs` (`18`), `math_train_7_algebra_substitution` (`11`), `bigbench_object_counting_instruments` (`3`) | E29 |
| `humaneval_0_has_close_elements`, `mbpp_2_similar_elements` | E30 |
| general text manipulation on arbitrary input (issue #244 requirement, not yet a benchmark case) | E31 |
| grow the suite past 5 cases + ratchet pass count (anti-memorization) | E32 |

---

## Parity Batch (E33-E34)

The fifth-pass 2026-05-29 audit, acting on the issue #244 PR (#245) feedback,
confirmed synthesis generality is built (benchmark suite passes 10/10) and found
the remaining gap is **parity**: "all Rust and JavaScript logic are in sync" and
"all languages are supported equally". These two epics close it without
re-implementing the now-general synthesis step — they generalize *how it is
triggered* (any supported language) and *where it runs* (Rust core and JS
worker), preserving the anti-memorization rule and the benchmark ratchet.

## E33 — Universal multilingual operation vocabulary (every handler triggers equally in en|ru|hi|zh) — [#326](https://github.com/link-assistant/formal-ai/issues/326)

Parent: #244. Part of the **parity batch (E33-E34)**. **Status: closed / merged (PR #328).**

**Problem.** Several reasoning handlers trigger only on English keywords even
though the agent advertises `supported_languages = en|ru|hi|zh`
(`data/seed/agent-info.lino`). `src/solver_handlers/text_manipulation.rs` matches
`uppercase`/`lowercase`/`reverse words`/`replace`/`sort lines`/`deduplicate
lines`/`count occurrences`/`count unique words`/`extract emails`, and
`src/solver_handlers/program_synthesis.rs` matches
`function`/`write`/`implement`/`return`/`count vowels`/`similar elements` — all
English-only. The operands these handlers consume are already language-neutral
(quoted segments, text after a colon, code identifiers), so only the operation
**verbs** are gated on English.

**Proposed approach.** Add a single shared, data-driven multilingual vocabulary
`data/seed/operation-vocabulary.lino` mapping each canonical operation token to
its localized synonyms for `en|ru|hi|zh`, mirroring how
`data/seed/intent-routing.lino` already carries multilingual keywords/phrases per
intent. Add a `seed::operation_vocabulary()` accessor and a canonicalization
helper that rewrites a normalized prompt's localized verbs to canonical English
tokens before the handlers match. This keeps the design **general, not specific**
— one vocabulary, many languages, extensible by editing data — instead of
per-handler, per-language literals. Mirror the seed into `src/web/seed/` via
`scripts/sync-seed.sh` so the browser shares the same data.

**Existing components to reuse.** `src/seed.rs` parser + `include_str!` registry;
`intent_routing()` as the structural model; the `supported_languages` test
pattern.

**Acceptance criteria.**
- `data/seed/operation-vocabulary.lino` exists with canonical → localized synonym
  entries for every supported language.
- `text_manipulation` and `program_synthesis` match on the canonicalized view;
  existing English specs are unchanged (no regression).
- Specification tests iterate `supported_languages` and assert each operation
  triggers in every supported language.
- No per-case memorization; operands remain derived from the prompt.
- Seed mirrored to `src/web/seed/`.

**Requirement links.** Issue #244 "all languages supported equally"; PR #245
feedback; `ROADMAP.md` Next Planning Batch (E33). A first increment lands in
PR #245; the full sweep across remaining English-only handlers is tracked here.

## E34 — Cross-runtime parity: JS browser worker mirrors Rust core synthesis (E28-E31) — [#327](https://github.com/link-assistant/formal-ai/issues/327)

Parent: #244. Part of the **parity batch (E33-E34)**. **Status: closed / merged (PR #329).**

**Problem.** The synthesis batch E28-E31 (#313-#316) added derivation paths to the
Rust core (general link-native synthesis, computed numeric/word-problem and
counting answers, program synthesis from spec + tests, generalized text
manipulation) that the JavaScript browser worker `src/web/formal_ai_worker.js`
has not yet absorbed. So the same prompt can be solved by derivation in Rust but
fall back to a weaker path in the browser, breaking "all Rust and JavaScript
logic are in sync" (vision pillar 18).

**Proposed approach.** Port the E28-E31 derivation paths into the worker so it
composes decomposed sub-results the same way the Rust core does, keeping
WebAssembly as the bridge for shared primitives and JavaScript as UI/glue. Add a
shared parity fixture set asserting both runtimes produce equivalent answers,
extending the E19 (#282) parity harness if present.

**Existing components to reuse.** `src/web/formal_ai_worker.js` solve pipeline and
WASM wrappers; the Rust `record_candidates` / handler logic as the reference; the
E19 #282 browser-worker parity work.

**Acceptance criteria.**
- The worker derives synthesis/numeric/program/text answers with the same
  algorithm shape as the Rust core.
- Shared parity tests assert equivalent answers from both runtimes.
- The anti-memorization rule and benchmark ratchet hold on the JS side.
- WebAssembly stays the primitive bridge; JavaScript stays UI/glue per pillar 18.

**Requirement links.** Issue #244 "all Rust and JavaScript logic in sync"; PR #245
feedback; `ROADMAP.md` Next Planning Batch (E34); vision pillar 18; mirrors E19
#282.

## Parity-batch coverage check

| Parity gap (code anchor) | Epic |
| --- | --- |
| English-only handlers (`text_manipulation.rs`, `program_synthesis.rs`) + new `operation-vocabulary.lino` | E33 |
| JS worker (`formal_ai_worker.js`) lacks E28-E31 derivation paths | E34 |
