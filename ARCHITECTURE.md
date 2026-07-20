# Architecture

This document describes the evolving architecture of `formal-ai`. Where
`VISION.md` captures *why* the project exists and `GOALS.md` captures *what*
counts as success, this document captures *how* the runtime is structured and
how each piece talks to the others. It is the canonical reference for new
contributors who want to understand the full pipeline without having to
triangulate between five other files.

Issue [#103](https://github.com/link-assistant/formal-ai/issues/103) names
this document as the single source of truth for the design and asks for the
following ideas to be explicit:

- last input + previous messages + memory + user data form the system context;
- the input is translated to Links Notation and recorded in memory before the
  formalization step;
- each verb phrase is formalized as a Wikidata P-ID and each noun phrase as a
  Wikidata Q-ID (with `wikipedia` / `wiktionary` URLs as fallbacks);
- multiple candidate interpretations are scored and selected with a
  neural-network-style **temperature** knob;
- close-probability candidates either ask the user a clarifying question or
  guess, depending on configuration;
- reasoning steps can nest, so tool-generated reasoning (e.g.
  `link-assistant/calculator`) is recorded as a sub-trace of the parent
  reasoning step;
- everything is appended to a growable memory backed by `doublets-rs` /
  `doublets-web`, with regular backups to browser storage and disk in `.lino`
  files;
- the local memory is treated as a cache of the public-knowledge database
  (Wikipedia, Wikidata, Wiktionary);
- the associative store supports stored transformation/substitution rules
  expressed as data, Rust/JS handlers, dynamically compiled Rust/JS, or
  natural-language skills convertible on demand;
- once an expression is formalized, the same engine translates between
  natural languages and between natural and programming languages.

The rest of this document expands each of those bullet points and links them
to the source modules that implement (or will implement) them.

---

## 1. System Context

The runtime computes an answer from four kinds of input:

1. **Last input message** — the raw text the user just sent.
2. **Previous messages** — the in-process conversation turns, expressed as
   `ConversationTurn { role, text }` (see `src/solver.rs`).
3. **Memory** — the append-only event log (see `src/memory.rs`,
   `src/event_log.rs`) plus the seed dataset under `data/seed/`.
4. **User data** — language preference, surface (chat / agent / CLI /
   Telegram / HTTP / browser), session preferences, and the
   `SolverConfig` knobs.

These are bundled into a `Context` object that is passed to the universal
solver. The solver reads context, never mutates it directly; mutations are
appended to the event log and the next request observes them through the
same Context construction step.

The Rust types involved:

- `formal_ai::ConversationTurn` and `formal_ai::ConversationRole` —
  conversation history.
- `formal_ai::MemoryStore` and `formal_ai::MemoryEvent` — durable memory.
- `formal_ai::ProbabilityStore` and `formal_ai::ProbabilityEvidence` —
  append-only symbolic probability evidence with provenance.
- `formal_ai::SolverConfig` — tunable knobs (`guess_probability`,
  `context_sensitivity`, `questioning_rigor`, `max_decomposition_depth`,
  `agent_mode`, `diagnostic_mode`, `offline`, `cache_ttl_seconds`).
- `formal_ai::seed::*` — the seeded knowledge (concepts, prompt patterns,
  intent-routing rules, multilingual responses, environment directory,
  identity card, tool registry).

Every surface (library, CLI, HTTP, Telegram, browser demo) assembles the
same `Context` shape so the same answer is produced regardless of how the
prompt arrived.

---

## 2. Pipeline Overview

```text
+-----------------------------------------------------------+
|                       1. INPUT                            |
|     (raw user message + history + memory + user data)     |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|                2. TRANSLATE TO LINKS NOTATION              |
|     - normalise text                                       |
|     - record raw impulse: as_is(impulse_NNN)               |
|     - parse into statement/question sequence               |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|                  3. RECORD IN MEMORY                       |
|        append impulse_NNN to event log (doublet links)     |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|                  4. FORMALIZATION                          |
|  verb phrases -> P-IDs (Wikidata properties)               |
|  noun phrases -> Q-IDs (Wikidata items)                    |
|  fallback: wikipedia / wiktionary URL                      |
|  emit candidate interpretations { P/Q/text, score }        |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|       5. TEMPERATURE-BASED INTERPRETATION SELECTION        |
|  rank candidates by score + probability evidence           |
|  apply temperature softmax                                 |
|  if top two are close:                                     |
|    - guess (when guess_probability is high), OR            |
|    - ask the smallest clarifying question                  |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|              6. UNIVERSAL PROBLEM SOLVER                   |
|  history lookup -> decomposition -> TDD synthesis ->       |
|  verification -> simplification -> presentation            |
|  may invoke sub-tools (calculator, JS, fetch) and nest     |
|  the tool's reasoning trace under the parent step          |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|                7. APPEND TO MEMORY                         |
|  every event is appended (impulse, candidate, validation,  |
|  source, trace, error, cache_hit, agent_action, etc.)      |
+----------------------+------------------------------------+
                       |
                       v
+-----------------------------------------------------------+
|             8. RENDER USER-FACING ANSWER                   |
|  + evidence links + Links Notation trace                   |
|  diagnostics gated by SolverConfig.diagnostic_mode         |
+-----------------------------------------------------------+
```

Each numbered step is implemented (or scheduled for implementation) as the
following Rust modules:

| Step | Module | Status |
| --- | --- | --- |
| 1. Input | `src/engine.rs::FormalAiEngine::answer` and `solve_with_history` in `src/solver.rs` | Implemented |
| 2. Translate to Links Notation | `EventLog::append("impulse", …)` in `src/event_log.rs` | Implemented |
| 3. Record in memory | `MemoryStore::append` in `src/memory.rs` | Implemented |
| 4. Formalization | `src/concepts.rs` plus `src/translation/formalization.rs` for scored P/Q-id, Wikipedia, Wiktionary, and raw fallback anchors | Implemented |
| 5. Temperature interpretation selection | `src/translation/selection.rs`, `src/probability.rs`, plus `SolverConfig::{temperature, guess_probability, questioning_rigor}` in `src/solver.rs` | Implemented |
| 6. Universal solver | `UniversalSolver` in `src/solver.rs` | Implemented |
| 7. Append to memory | `event_log::EventLog`, `memory::export_full_memory` | Implemented |
| 8. Render user-facing answer | `SymbolicAnswer` projection in `src/engine.rs` | Implemented |
| 9. Natural-language skill compilation | `src/skill_compiler.rs` plus the `behavior_rules` replay bridge, and `src/skill_procedure.rs` plus `src/solver_handlers/procedure_rules.rs` for freely phrased procedures | Implemented for deterministic trigger/response skill packages and for multi-step procedures stated in ordinary prose |

The pipeline runs the same way for every prompt — greetings, identity,
concept lookup, math, code generation, idioms, refusals, agent actions —
because the universal solver is intentionally domain-agnostic. Specialized
handlers (`solver_handler_units`, `solver_handler_how`, `solver_handlers`,
`solver_handlers_policy`) are *plugged into* the universal solver, not
branched on by domain at the top level.

Within step 6, the solver tries its specialized handlers in a fixed
first-match-wins **precedence order**. That order is *data*, not code: it lives
in `data/seed/handler-precedence.lino` as an ordered list of bare handler-name
rows, each optionally carrying a trailing `#` guard note (issue #663, "Data Is
The Interface"; the name is the row head, so the seed's meaning-closure audit,
which grounds only *value* tokens, leaves the precedence table alone).
`src/solver_dispatch.rs` keeps only
the executable function pointers (`HANDLER_FUNCTIONS`, which must stay Rust) and
`specialized_handlers()` joins the two — asserting at load time that the seed is
an exact permutation of the registry, so a seed edit can never silently drop or
duplicate a handler. Reordering two rows in the seed changes routing (proven by
`cargo test routing_precedence_from_seed`); the shipped seed preserves today's
behaviour. `MethodRegistry::from_dispatch` (`src/method_registry.rs`) surfaces
this seed-ordered table as the `Specialized` method surface, and
`meta_method_dispatch::try_dispatch` consumes it. The browser worker mirrors the
seed through `src/web/seed_loader.js`; because it names its handlers differently
and runs its async fetch handlers in a later phase, full order-parity is
impossible, so `tests/fixtures/routing-parity.lino` pins the *shared* precedence
invariants both surfaces must honour (checked by the routing-parity test).

The precedence is also something Formal AI re-derives *itself*, through its own
Agent CLI: the rationale behind the ordering (`#395`, `#423`, `#425`, `#552`,
http_fetch-first, incompatible_units-last) is a persisted associative links
network (`data/meta/issue-663-handler-precedence-learning.lino`) that the
`handler_precedence_learning` report ranks into a human-review-gated proposal.
The report is one row in the `REPORTS` table
(`src/agentic_coding/learning_report.rs`) — data-routed, not a planner branch —
and its committed evidence is byte-for-byte reproducible by the in-process
renderer (`tests/unit/issue_663_handler_precedence_learning.rs`), so the tool,
not a hand-edit, is the author. See `docs/case-studies/issue-663/`.

---

## 3. Translating Input to Links Notation

The chat surface stores the raw impulse as a Links Notation link first, then
optionally re-parses it into a sequence of statements or questions. The
canonical shape is:

```text
impulse_0042
  as_is "Write me hello world in Rust"
  language "en"
  surface "cli"
  ts 1747488000
  user "anonymous"
```

The `as_is` field is required and is the ground truth — every later step is
allowed to fail or be revised, but the original message is never rewritten.
This matches the **add-only history** principle from `VISION.md`.

Multi-statement prompts are split into a `link statements` block:

```text
impulse_0042
  as_is "Hi! Translate fn add to Python."
  statements
    statement "Hi!"
    statement "Translate fn add to Python."
```

Splitting is best-effort: when it is ambiguous, the solver records every
candidate split as its own `candidate` event and lets the temperature step
pick.

---

## 4. Memory: Doublet Links, .lino Backups, and the Public-Knowledge Cache

Memory has three layers:

### 4.1 Local in-process event log

Implemented today by `src/event_log.rs` and `src/memory.rs`. Normal writes are
content-addressed (FNV-1a 64-bit) and appended. The log is exposed through
`SymbolicAnswer::evidence_links` (short, user-visible) and
`SymbolicAnswer::links_notation` (full trace).

Issue #196 adds explicit destructive maintenance paths on top of that normal
append-only model. `MemoryStore::purge_deleted_conversations` physically
removes events for conversations that already have a `conversation_deleted`
marker, `MemoryStore::purge_conversation` removes one conversation by id, and
`MemoryStore::reset` clears the dynamic event log. Browser IndexedDB mirrors
the same split with selected cursor deletion and full object-store `clear()`.
User-facing surfaces guard these operations with an export-first path and an
irreversible confirmation, while the CLI requires `--confirm` and can write a
full-bundle `--backup` before modifying the memory file.

### 4.2 Dreaming maintenance planner

Issue #540 adds `src/dreaming.rs`, a default-on, low-priority maintenance
planner over the same `MemoryEvent` projection. Dreaming reads memory and emits
an inspectable plan; it does not mutate memory unless the caller explicitly
uses `formal-ai memory dream --apply --confirm`, with the same optional
full-bundle `--backup` as reset and purge-deleted.

The planner classifies events into five `DreamingDurability` classes:

- `IrreplaceableRaw` for raw user/assistant/system experience;
- `RetainedLearning` for learning ledgers, promoted lessons, generalized
  algorithms, and baked-in meta-algorithm amendments;
- `DeletedConversation` for data already attached to a soft-deleted thread;
- `RecomputableCache` for public-source cache and fetch/tool output;
- `RecomputableIntermediate` for derived summaries and conclusions.

Only deleted conversation data, recomputable cache data, and recomputable
intermediate data are reclaimable. Duplicate cleanup is limited to the
recomputable classes and recalculates usage by scanning current event text and
evidence links before deciding which duplicate to keep. Under storage pressure,
`DreamingConfig` targets a 20% free-space reserve by default, subtracts the
next known `incoming_bytes`, and selects the lowest-use reclaimable records
first. If reclaimable records cannot satisfy the target, the plan reports
`requires_bigger_storage` instead of selecting raw or learned experience.

Dreaming also *learns from memory links and generalizes*. `event_topic` ranks
frequent topics, `requirement_statement` reads multilingual cues from
`data/meta/dreaming-cues.lino`, and `mine_patterns` derives recurring task
structures directly from records. Proposed `MetaAlgorithmAmendment` values are
replayed against discovered candidate tasks; only an exact normalized replay
may mark a specific as covered. Applied amendments are retained as structured
`meta_algorithm_amendment` events, and `src/dreaming_application.rs` reads those
events on later OpenAI-compatible requests so the learned rule changes similar
future answers without being repeated. Under pressure, only replay-verified
specifics can be forgotten via `ForgetCoveredSpecific`.

`src/storage_policy.rs` measures actual filesystem capacity/free bytes and the
next incoming write. Automatic removal requires a persisted `.auto-free-space`
choice; both CLI and Electron can ask, and Electron warns when larger storage is
still required. `src/dreaming_runtime.rs` runs the same learning loop in the
core server, guarded by foreground activity, while Electron additionally uses
system-idle detection and lowest practical cross-platform process priority.
The thirteen-stage recipe in `data/meta/dreaming-recipe.lino` is pinned to all
of these live source modules.

The Electron desktop shell starts `desktop/lib/dreaming.cjs` by default as a
plan-only background task. It waits before its first run, repeats infrequently,
unrefs timers/processes, and wraps the CLI with `nice -n 19` on Unix-like
platforms. Operators can disable that scheduler with
`FORMAL_AI_DESKTOP_DREAMING=off`.

### 4.3 Default native doublets-rs / doublets-web store

Native Rust builds select `LinkStoreBackend::DoubletsRs` by default because
Cargo's default feature set enables `doublets-native`. The library exposes
`link_store::DefaultNativeLinkStore` and `default_native_link_store()` so
embedders can construct the active native backend without checking feature
flags themselves. Compiling with `--no-default-features` keeps the explicit
`MemoryStore` / `.lino` projection fallback for small builds and recovery
tools.

The native backend mirrors each `MemoryEvent` into the `doublets-rs` links network using
the `Type -> SubType -> Value` reduction in `src/link_store.rs`. Links
Notation remains the deterministic projection for inspection, backup,
recovery, and migration: `import_memory_links_notation` accepts both legacy
`demo_memory` files and full `formal_ai_bundle` exports, while malformed
documents are rejected before the store is mutated. Exporting the native
store writes the same stable `.lino` event log that the CLI, HTTP, Telegram,
and browser surfaces use for portability.

Browser storage remains compatible with `doublets-web`: `src/web/memory.js`
uses IndexedDB for the event object store, reports `doublets-web` when a
browser doublets implementation is available, and otherwise keeps the
`indexeddb-lino-mirror` fallback. The browser and native stores therefore
share Links Notation import/export semantics even though their physical
storage engines are different.

Upstream references:

- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs)
- [`linksplatform/doublets-web`](https://github.com/linksplatform/doublets-web)

Implemented migration surface:

1. Wrap the current memory projection in a trait so the active backend is
   swappable (`link_store::LinkStore`).
2. Enable the `doublets-rs` backend by default for native builds through
   `doublets-native`.
3. Preserve `--no-default-features` as the explicit `.lino` projection
   fallback.
4. Mirror native writes to `.lino` snapshots via
   `memory::export_links_notation`.
5. Accept existing `.lino` memory files and full bundles as migration input.
6. Keep the browser IndexedDB/doublets-web mirror on the same projection
   contract.

### 4.4 Public-knowledge cache

When the local memory does not contain enough evidence to satisfy a prompt,
the solver follows the **source cache protocol** (see
`tests/unit/specification/source_cache.rs`):

- check the local `source_cache` for an entry under
  `source:wikipedia:<lang>:<slug>` (or `source:wikidata:<P|Q-id>` / 
  `source:wiktionary:<lang>:<word>`).
- if absent and `offline` is false, fetch the external source and record a
  `source:` event with `fetched_at` and `sha256`.
- if `offline` is true, refuse the fetch and record a `policy:offline`
  event.

Every external fetch ages out after `cache_ttl_seconds` (default ≈ 60 days).
This is the architectural answer to "instead of GPU and neural networks, use
reasoning with internet as a public database with our local memory as
cache."

### 4.5 Fact-query reasoning pipeline (Issue #127)

Structured factual prompts — "what is the capital of France?", "столица
Германии", "भारत की राजधानी", "中国的首都" — are answered by a dedicated
reasoning pipeline that combines the seed cache with live Wikidata calls.
The pipeline is implemented in `src/web/formal_ai_worker.js` as
`parseFactQuestion` + `tryFactQuery` and mirrored in Rust as
`src/solver_handlers/benchmark_prompts.rs::try_fact_lookup` (the offline
solver uses the seed exclusively; the browser worker reaches the live API
on cache miss).

Pipeline stages:

1. **Parse.** `parseFactQuestion(prompt, normalized)` extracts a
   `(relation, subjectTerm, language, forceFresh)` tuple. The relation
   slug — `capital`, `population`, `currency`, `official_language`,
   `continent`, `area`, `head_of_state`, `head_of_government` — anchors to
   a Wikidata property (`P36`, `P1082`, `P38`, `P37`, `P30`, `P2046`,
   `P35`, `P6`). Multilingual regexes recognize the question in en/ru/hi/zh.
2. **Cache check.** A 1-week TTL in-memory store is keyed by
   `<relation>|<subjectTerm>|<language>`. `data/seed/facts.lino` entries
   that carry a `relation` field pre-warm the cache at worker startup via
   `warmFactCacheFromSeed`, so every seeded country resolves offline. The
   user can opt out of the cache with force-fresh markers in any supported
   language (`refresh`, `не из кэша`, `ताज़ा`, `刷新`, …).
3. **Wikidata resolution.** On cache miss the worker calls
   `wbsearchentities` to map the subject term to a Q-ID, then
   `wbgetentities` to read the relation's property claim and its label /
   sitelink in the user's language.
4. **Cache store.** The resolved triple `(subject_qid, value_qid, summary,
   source_url)` is written back to the cache with the original
   `fetched_at` timestamp.
5. **Trace.** Every step is appended to the event log as a `fact_query:*`
   event (`fact_query:request`, `fact_query:relation`,
   `fact_query:subject`, `fact_query:cache:check`, `fact_query:cache:hit`,
   `fact_query:cache:miss`, `fact_query:wikidata:*`, `fact_query:response`)
   so the reasoning trace can be reconstructed from memory.

The Rust offline solver follows the same shape: when a `fact_*` record in
`data/seed/facts.lino` declares a `relation`, the matcher emits the
structured `fact_query:relation`, `fact_query:subject`,
`fact_query:cache:hit:seed`, `fact_query:subject_qid`, and
`fact_query:value_qid` events alongside the legacy `fact_lookup:*` events.
That guarantees the Rust and browser stacks agree on the evidence shape
even though only the browser stack reaches Wikidata at runtime.

---

## 5. Formalization

Formalization converts free-form text into typed link references. The
target shape:

```text
formalization
  subject_q   "wikidata:Q14660"     # noun phrase -> Q-id
  predicate_p "wikidata:P31"        # verb phrase -> P-id
  object_q    "wikidata:Q170978"
  source_text "is a sorting algorithm"
  language    "en"
```

Where a P/Q-id does not yet resolve, the formalizer falls back to a
`wikipedia:` or `wiktionary:` URL. The fallback chain is:

1. Wikidata item / property (fully formal, language-independent).
2. Wikipedia article (per-language; bridges to Wikidata through `Q-id` if
   one exists for the article).
3. Wiktionary entry (per-language; useful for verbs and idioms that
   Wikidata does not model).
4. Raw text in `as_is` only, with a `formalization_unresolved` flag.

The formalizer is deliberately allowed to emit **multiple** interpretations
per phrase. Selection happens in step 6.

The current implementation has two cooperating formalization layers:

- `src/concepts.rs` handles seed concept lookup through explicit aliases and
  context hints.
- `src/translation/formalization.rs` handles arbitrary prompt fragments with
  a deterministic multilingual label table, concept-seed Q-id reuse, scored
  Wikidata P/Q anchors, and explicit Wikipedia/Wiktionary/raw fallbacks.

`src/solver.rs` records the selected formalization as `formalization:*`
events before local search, including typed links such as
`formalization:predicate_p:wikidata:P31`,
`formalization:subject_q:wikidata:Q89`, and
`formalization_unresolved:<surface>` for later translation-gap handling.

---

## 6. Temperature-Based Interpretation Selection

Each candidate formalization carries a `score` field. The solver normalizes
the scores by a softmax controlled by the same temperature knob a neural
network would use:

```text
P(c_i) = exp(score_i / T) / Σ exp(score_j / T)
```

- `T = 0`  → deterministic; the highest-scored candidate always wins.
- `T = 1`  → maximum configured exploration across the scored candidates.

The temperature is sourced from `SolverConfig`. `src/translation/selection.rs`
normalizes 0..1000 formalization scores to 0.0..1.0, applies a stable
softmax, and uses a content-hash-seeded draw whenever the solver must guess
under ambiguity. This keeps the same prompt + same config deterministic.

If the top-two probabilities are within ε (configurable through
`SolverConfig.questioning_rigor`), the solver:

- if the configuration permits guessing, samples from the softmax distribution
  with the impulse hash as a seed and records a
  `policy:guessed_under_ambiguity` event so the trace is honest;
- otherwise, emits a clarifying-question intent (the smallest question that
  separates the candidates) and stops the pipeline until the user replies.

The seeded-from-impulse-hash draw in `src/translation/selection.rs` keeps
guessing deterministic per prompt, so the same input + same config produces the
same answer.

---

## 6.1 Symbolic Probability Evidence

Issue #279 adds a narrow probabilistic layer without changing the project's
non-neural boundary. `src/probability.rs` stores evidence as ordinary
append-only records:

```text
probability_evidence
  id "probability_..."
  target "formalization:subject=wikidata:Q89 predicate=wikidata:P279 object=wikidata:Q3314483"
  observation "taxonomy_context_prefers_subclass"
  weight "1.000000"
  model "bayesian_evidence"
  provenance "source:seed:test"
  recorded_at "2026-05-26T00:00:00Z"
```

The current supported models are deliberately small:

- `bayesian_evidence` adds independent symbolic evidence weights to a
  candidate's prior score before temperature softmax.
- `markov_transition` applies a weight only when the previous symbolic state
  matches `ProbabilityRankingConfig::markov_from`.

Both models operate on symbolic target IDs, not neural logits. The selector
still produces a deterministic `FormalizationSelection`: the same prompt, same
probability store, same config, and same impulse hash produce the same selected
candidate. Evidence records can carry `source_url`, `fetched_at`, `sha256`, and
`cached` fields; offline mode ignores live-only evidence, preserves cached
source provenance, and emits `policy:offline` for skipped live evidence.

The solver exposes `UniversalSolver::solve_with_probability_store` for callers
that have a probability store. The default `FormalAiEngine::answer` path uses
an empty store, so existing deterministic behavior is unchanged until evidence
is explicitly supplied.

### Evidence count and counted-utility ranking (issue #449)

Issue #449 ports the interpretable, non-neural mechanisms from Kolonin's
"Interpretable Experiential Learning" (arXiv:2605.00940) onto this same
associative layer. The paper models behaviour as a transition graph where every
transition carries both a **utility `U`** and an **evidence count `C`**; the
existing store already accumulated `U` (`target_weight`) but folded the count
into it. The additions keep the two separate without leaving the non-neural
boundary:

- `ProbabilityStore::target_evidence_count` returns the count `C` of append-only
  observations supporting a target, using the same offline and Markov-state
  filters as `target_weight`, so `U` and `C` always describe the same evidence
  subset.
- `ProbabilityRankingConfig` gains three opt-in fields mirroring the paper's
  decision-policy hyperparameters: `counted_utility` (`CU` — rank by
  `argmax(U·C)` instead of `argmax(U)`), `min_transition_utility` (`TU`), and
  `min_transition_count` (`TC`). A transition below `TU`/`TC` is treated as
  under-evidenced: its learned evidence is withheld and the candidate falls back
  to its structural prior.
- `RankedProbabilityCandidate` exposes `evidence_count` and `similarity` next to
  `evidence_weight`, so each ranked option is locally interpretable — it carries
  the utility, the number of observations behind it, and how the evidence was
  matched.

The defaults (`counted_utility = false`, both thresholds `None`) reproduce the
prior additive behavior, which equals the paper's recommended `CU=False`,
`TU=0`, `TC=1` baseline; existing callers are unaffected until they opt in.

#### Similarity fallback (`SS`)

The paper falls back to the **closest** stored state when no exact transition
matches, gated by a cosine-similarity floor `SS`. The same idea is ported
symbolically:

- `symbolic_cosine_similarity` computes a deterministic bag-of-words cosine over
  the alphanumeric tokens of two target IDs — no embeddings, no neural logits.
- `ProbabilityStore::nearest_similar_evidence` finds the highest-similarity
  stored target (above the `similarity_threshold` floor, ties broken by target
  name) and lends its evidence, scaled by the similarity, to a candidate that
  has no exact evidence of its own.
- The fallback only fires when the candidate's direct evidence count is zero,
  and the borrowed evidence still passes the `TU`/`TC` gate, so an
  under-evidenced neighbour cannot smuggle weight past the thresholds.

#### Episode-wide global feedback

`ProbabilityStore::reinforce_transition_path` mirrors the paper's one-shot,
episode-wide reward: given an ordered state path and a reward, it appends one
`markov_transition` observation per adjacent pair, so a whole successful episode
reinforces every transition it traversed in a single append-only pass. The
records replay deterministically through the event log and link-store projection
like any other evidence.

#### Generalized decision policy

The `CU`/`TU`/`TC`/`SS` knobs are grouped into a single
`ProbabilityDecisionPolicy` value. `SolverConfig::probability_policy` threads it
through every selection use case — `select_formalization_candidate_with_policy`
(the formalization selector) and `try_synthesize_from_sub_results` (the
synthesis ranker) — via `ProbabilityRankingConfig::with_decision_policy`. The
default policy is the paper's baseline, so the store-only entry points delegate
with it and existing surfaces are byte-for-byte unaffected until a caller opts
in. The full analysis is archived in `docs/case-studies/issue-449/`.

---

## 7. Universal Problem Solver

The solver follows the universal loop documented in `VISION.md` (Section
"Universal Problem-Solving Algorithm"). The implementation is in
`src/solver.rs`:

1. **Impulse** — an `impulse` event is appended through `EventLog::append`.
2. **Formalization** — alias resolution plus P/Q-id lookup with fallbacks.
3. **Context and domain data** — language detection, surface, mode flags.
4. **History lookup** — search local doublets first; record `cache_hit`
   on success.
5. **Decomposition** — split conjunctions ("and", "with tests", "with
   benchmarks") into sub-impulses.
6. **TDD-style test generation** — emit at least one `test:` event per
   candidate.
7. **Solution synthesis** — reuse known parts → reason from rules →
   randomized / evolutionary search if the structure allows.
8. **Combination** — recombine partial solutions.
9. **Verification** — run candidate against generated tests; surface
   `trace:execution_failure` on failure.
10. **Simplification** — apply meaning-preserving transformation rules to
    shrink the answer.
11. **Presentation** — produce the user-facing reply + Links Notation trace
    + evidence links.

Every numbered step writes its own event before the next one starts.

### 7.1 Project lookups and summarization

"What is `<project>`?" prompts about projects and repository URLs go through a
generic `project_lookup` path. When associative project promotion is enabled
(the default), repositories from `link-assistant`, `link-foundation`, and
`linksplatform` are listed first when they match the prompt; turning promotion
off keeps the same prompt on the generic GitHub/GitLab/Bitbucket lookup path.
The pipeline has three pieces:

1. **Curated registry.** `data/seed/projects.lino` records the canonical
   repository, primary language, weighted statements, English/Russian
   localisations, topic label, and aliases for each project. The seed file is
   embedded at compile time and parsed once per process via
   `src/seed/projects.rs::projects_registry()`.
2. **Formalize → summarize → deformalize pipeline.** `src/summarization/mod.rs`
   exposes a deterministic three-stage pipeline. `formalize` (or
   `Statement::from_seed`) turns free-form prose or curated statements into a
   homogeneous `Vec<Statement>` with a `StatementKind` (identity, purpose,
   language, stars, feature, use_case, install, example, misc) and a numeric
   weight. `summarize` then applies a `SummarizationConfig` whose
   `SummarizationMode` selects the target size — `Topic` (1–5 words),
   `Short` (~20%), `Standard` (~50%), `Full` (100%), or `Expand` (~200%) — and
   the optional explicit `max_statements` cap. Boilerplate kinds (`install`,
   `example`) are dropped from compressed answers; `Expand` mode appends
   Natural Semantic Metalanguage paraphrases. `deformalize` joins the
   surviving statements back into a single block of prose.
3. **Handler integration.** The solver dispatch table in `src/solver.rs`
   still lets `concept_lookup` answer seed concepts such as Links Notation
   first. Immediately after a concept miss, `project_lookup` handles promoted
   project aliases such as Hive Mind or link-cli, explicit GitHub/GitLab/
   Bitbucket repository URLs, and the promotion-off fallback. Promoted answers
   log `project:promoted`, `summarization:mode`, `summarization:language`, the
   repository URL, and the web-search providers consulted alongside the local
   answer so the trace explains both *what* was matched and *how* the text was
   compressed.

The compression knobs are configurable from one struct (`SummarizationConfig`)
so callers can dial topic labels, chat titles, project descriptions, or
expanded explanations from the same pipeline.

The same pipeline also drives four additional surfaces:

- **README ingestion.** `strip_markdown_noise` removes badges, fenced code
  blocks, HTML comments, heading markers, and blockquote chevrons. The
  cleaned prose is fed through `formalize_markdown` (a thin wrapper around
  `formalize`) and `describe_readme(repo_slug, markdown, &config)`. In
  `Topic` mode the helper returns the repository slug so the same call can
  serve as a chat-title source for a fetched repository.
- **Repository-file summaries.** `formalize_repository_file(path, content)`
  detects common repository file formats, records path/format/line/byte
  metadata, converts file content into ranked statements, and renders the
  result as link-native `repository_file` notation. Supported source and data
  grammars also carry `MetaLanguageFormalization` evidence from
  `meta_language::LinkNetwork` (parser label, syntax-link count, total-link
  count, parse-error state, and text-preservation state). Markdown files are
  formalized recursively: prose is summarized through `formalize_markdown`, and
  each fenced code block becomes an `EmbeddedGrammarFormalization` with its own
  normalized language label and optional parser evidence. `summarize_repository_file`
  then reuses `SummarizationConfig`, `summarize`, and `deformalize` so file
  summaries follow the same modes and caps as project, README, and dialog
  summaries.
- **Repository-resource summaries (files and folders).**
  `src/summarization/resource.rs` generalizes file summarization to any
  repository resource so the solution is not file-specialized. A caller builds a
  filesystem-free `RepositoryEntry` tree (`RepositoryEntry::file` /
  `RepositoryEntry::directory`); `formalize_repository_resource` then dispatches
  on kind, reusing `formalize_repository_file` for files and recursing through
  `formalize_repository_directory` for folders. A directory is summarized by the
  meta algorithm's decompose -> summarize -> compose loop: it is split into its
  children, each child is summarized on its own (files via `file.rs`,
  subdirectories by recursion), and the child summaries are composed behind an
  aggregate identity sentence carrying recursive file/subdirectory counts and
  total lines/bytes. Recursion depth is bounded by the *mode ladder*
  (`SummarizationMode::one_step_shorter`): a `Full` folder describes its direct
  children in `Standard`, theirs in `Short`, and everything deeper as a `Topic`
  label, so arbitrarily deep trees stay bounded while the most important
  structure surfaces first. `RepositoryDirectoryFormalization::links_notation`
  renders a link-native `repository_directory` block (path, counts, per-child
  kind) for inspectable evidence, and `summarize_repository_resource` is the
  general entry point that subsumes `summarize_repository_file` for file inputs.
- **Dialog summarization.** `DialogTurn { role, text }` and
  `formalize_dialog` weight user turns +20 and assistant turns -10 so a
  short summary keeps the user's questions even when both sides talk a
  lot. `summarize_dialog(turns, &config)` runs the result through
  `summarize` / `deformalize`, and `generate_chat_title(turns, language)`
  wraps it in `SummarizationMode::Topic`. `try_summarize_conversation` in
  `src/solver_handlers/mod.rs` now collects `prior_turn:user` and
  `prior_turn:assistant` events into `DialogTurn`s, calls `summarize_dialog`
  in `Standard` mode, and logs `summarization:mode`,
  `summarization:language`, and `chat_title` evidence alongside the
  per-turn list.
- **HTTP fetch for curated GitHub URLs.** When `try_http_fetch` recognises
  a `github.com/<org>/<name>` URL whose `<org>/<name>` matches the curated
  registry (`match_curated_github_url`), the handler runs `describe_project`
  in `Standard` mode and embeds the result in the response. The trace
  records `http_fetch:curated_project`, `summarization:mode`, and
  `summarization:language` so the path from URL → curated record → summary
  is fully visible.

The shared `DEFAULT_MAX_STATEMENTS = 30` constant in `src/summarization/mod.rs`
documents the default cap on retained statements; any caller can raise or
lower it with `SummarizationConfig::with_max_statements`.

---

## 8. Nested Reasoning Steps

Tools called during synthesis can produce their own reasoning steps. The
calculator (`link-calculator`) is the current canonical example: when the
universal solver decides to delegate a calculation, the calculator's own
`StepRecord` items are appended to the parent trace as a **nested** sub-trace
under the parent `candidate:` event.

```text
candidate_007
  intent calculation
  expression "8% of $50"
  delegate "link-calculator"
  nested
    step "extract percentage 8"
    step "extract amount 50 USD"
    step "compute 4 USD"
  result "4 USD"
```

The nested-trace contract holds for every future tool integration (HTTP
fetch, Wikipedia summary, JS execution, etc.). Each tool returns a
`Vec<NestedStep>` instead of an opaque value, so the user can ask "why?" and
get a step-by-step explanation at any depth.

---

## 9. Transformation and Substitution Rules

The associative store supports five kinds of rules. They are listed in
order from "lowest privilege" to "highest privilege":

1. **Pure data rules** — `when LHS then RHS` doublet patterns. No code is
   executed; the rule rewrites links in place. Stored as Links Notation,
   reviewable by a human.
2. **Rust handlers** — compiled-in Rust functions registered against a rule
   id. Today this is how the specialized handlers in `solver_handlers/`,
   `solver_handler_units.rs`, and `solver_handler_how.rs` are wired up.
3. **JavaScript handlers** — JS functions registered for the browser worker
   and the upcoming `try_javascript_execution` solver step. They run in a
   sandboxed Worker.
4. **Dynamically compiled Rust/JS** — code snippets stored *inside* the
   associative store as text and compiled (Rust) or interpreted (JS) on
   demand. The compiled output is cached by the snippet's content hash.
5. **Natural-language skills** — prose instructions like "When the user asks
   for hello world in language X, look up the seed for X and print the
   verified output." These are compiled on demand into one of the above
   four representations: a data rule, a Rust handler stub, a JS handler
   stub, or an interpreted sequence of solver steps.

`src/skill_compiler.rs` implements the deterministic compiler subset. The
legacy `When ... answer ...` form still lowers into a `CompiledSkillPackage`
with a trigger rule, a deterministic compiled handler, an E1-style
`LinkRecord` projection, and a Links Notation export. The structured subset
adds reviewable `Skill`, typed `Input`, `Precondition`, ordered `Step`,
`Effect`, `Expected test`, `Permission`, `Tool`, and `Target` records. Expected
tests become deterministic replay fixtures, and `Target` records produce
inspectable Rust/JavaScript/native handler stubs rather than executable code.
The compiler refuses unsupported or nondeterministic instructions and requires
explicit `Permission` records for package/tool capabilities such as
`tool:local_shell`. The solver scans dialog history for compiled packages
before falling back to behavior-rule re-derivation; a replay appends
`compiled_skill:replay` and `cache_hit:<compiled_skill_id>` to the trace.

`src/skill_procedure.rs` covers the prose that falls outside that typed shape
(E55, issue #674). It splits a request such as "when I paste a link, fetch its
title, translate it to Russian, save both, and reply with the translation" into
ordered clauses and maps each clause onto a step verb seeded in
`data/seed/meanings-skill-procedure.lino`, so the vocabulary grows as data. Two
guards keep ordinary prompts out: the request needs a seeded trigger lead and at
least two recognized steps. The compiled program is projected from canonical
slugs only, so the English, Russian, Hindi, and Chinese phrasings of one
procedure content-address to the same id and the same `LinkRecord`s. Each step
keeps the source sentence span it was read from, which is what
`src/solver_handlers/meta_explanation.rs` quotes when asked *"why did you do
that?"*. A clause with no vocabulary entry compiles nothing at all:
`src/solver_handlers/procedure_rules.rs` answers with the named gap and appends a
`skill_gap` event rather than dropping the step.

`src/associative_package.rs` is the R65 package boundary. It models
Deep.Foundation-inspired packages in the local doublet architecture with
package metadata, dependency links, handler records, trigger records, and
explicit permission grants. A package can be exported/imported as Links
Notation, installed only after dependencies validate, replayed through its
trigger/handler links, and queried by the tool-call gate for capabilities such
as `tool:calculator`. Compiled skills can be wrapped as packages and imported
back without hand-editing Rust code; structured expected tests become package
triggers/handlers and structured permissions become package permission grants.
The `/api/formal-ai/v1/network` projection (the deprecated `/api/formal-ai/v1/graph`
alias still resolves) includes the package, handler, trigger, and permission
links so the permission path is inspectable alongside ordinary rules.

The compilation chain (NL → code → binary) is the long-term path. The
runtime never *requires* compilation: a natural-language skill can be
interpreted one step at a time without ever being lowered to Rust/JS.

---

## 10. Translation Between Languages

Because formalization is language-independent (a Wikidata Q-id is the same
whether it is named in English, Russian, Hindi, or Chinese), translation is
not a separate model — it is a re-rendering of the same formalized links network
into the target language's labels.

```text
formalization_007
  subject_q "wikidata:Q170978"
  predicate_p "wikidata:P31"
  object_q "wikidata:Q14660"

render_en  -> "QuickSort is an instance of sorting algorithm"
render_ru  -> "Быстрая сортировка — это разновидность алгоритма сортировки"
render_hi  -> "क्विकसॉर्ट एक छँटाई एल्गोरिथ्म का उदाहरण है"
render_zh  -> "快速排序是一种排序算法"
```

The same machinery translates between natural and programming languages.
When the formalizer recognizes the input as a programming-language
construct (Rust function, Python class, SQL query), it lifts the construct
to a formalized links network the same way and re-renders into any other language
the renderer supports. The renderer is a transformation rule (Section 9):
the input is `(links_network, target_language)`; the output is rendered text.

### 10.1 Formalize → Meaning → Deformalize Pipeline

Translations flow through a generalized pipeline that resolves any
surface pair via existing public knowledge bases — Wiktionary's
translation tables and Wikidata's lexeme/sense graph — instead of a
hand-written list of phrase pairs:

```text
formalize(surface, source_lang)
  -> Wiktionary translation_blocks(source_edition, page)
  -> sense_blocks ⨯ candidates(target_lang)
meaning(source_lexeme, target_lexeme)
  -> Wikidata SPARQL: ?lexeme ontolex:sense ?sense .
                      ?sense  wdt:P5137     ?meaning .
  -> MeaningId (priority: Q-item > sense > Wiktionary page)
deformalize(meaning, target_lang)
  -> winning candidate by round-trip confirmation
match_source_formatting(target, source)
  -> mirrors the source fragment's leading capitalization
     and terminal punctuation
```

The pipeline lives under `src/translation/`:

- `src/translation/http.rs` — `HttpClient` trait. The default
  transport shells out to `curl` so the crate has no TLS dependency.
- `src/translation/cache.rs` — `CachedHttpClient` persists raw
  response bodies under `data/translation-cache/<fnv1a>.body` with a
  sibling `.url` file. Online mode is gated by `FORMAL_AI_LIVE_API`;
  offline mode reads only from the committed cache, so every test
  runs deterministically.
- `src/translation/wiktionary.rs` — parses `{{t|...}}` / `{{t+|...}}`
  / `{{tt|...}}` / `{{перев-блок|...}}` / `{{翻譯-頂}}...{{翻譯-底}}`
  templates and splits polysemous entries by `{{trans-top|gloss}}`
  blocks.
- `src/translation/wikidata.rs` — runs the canonical lexeme join
  (`ontolex:sense` / `wdt:P5137`) so two surfaces share a stable
  `meaning:` id regardless of which language we observe first.
- `src/translation/meaning.rs` — `MeaningId` selector.
- `src/translation/pipeline.rs` —
  `TranslationPipeline::translate(surface, source, target)`.
- `src/translation/formatting.rs` — `match_source_formatting` keeps
  lowercase phrases lowercase, capitalizes targets when the source
  fragment is capitalized, and only emits a terminal `? ! .` (or the
  Chinese full-width equivalents `？ ！ ．`) when the source carried
  one.

The meaning ID, source language, and target language remain in
`evidence_links` so the Links Notation trace is still inspectable;
the user-facing body is just the deformalized surface.

### 10.2 Resolution Order and Browser Fallback

`TranslationPipeline::translate` resolves any surface pair by:

1. Fetching the source-edition Wiktionary page and parsing its
   `{{trans-top}}` blocks for `target_lang` candidates.
2. Falling back to the `/translations` subpage when the main page
   omits translations (common for high-traffic English entries).
3. Falling back to the target-edition Wiktionary page in reverse when
   the source edition is sparse (typical for ru → en).
4. Generating phrasal variants (e.g. dropping Russian "у тебя",
   "у вас", "у меня" infixes) when the literal page does not exist.
5. Selecting the best sense block by round-trip confirmation rate —
   for each candidate, count how many target-edition pages list the
   source surface as a translation. The block with the most confirms
   wins.
6. Upgrading the meaning id to a Wikidata Q-item or sense id when the
   lexeme join returns one.

Issue #526 promotes that round-trip confirmation from a ranking heuristic into
the testable quality contract. The active matrix requires
language-to-meta-to-same-language survival for every supported natural language
and a directed pair round trip across en, ru, hi, and zh. Code translation uses
the same shape and the same anti-`N * N` rule: `translate_program` never matches
on a `(source, target)` pair. It formalizes source code into a language-neutral
`CodeMeaning` (`src/solver_helpers/code.rs::formalize_code_meaning`) and renders that
meaning into the target (`render_code_meaning`), so adding a language is one
formalizer plus one renderer, not a new pair. Because the source language never
enters the formalizer, any pair — including ones with no hardcoded arm, such as
Python -> JavaScript — shares one meaning, and Rust <-> JavaScript returns to the
same `meaning:` link.

The Rust pipeline is the canonical implementation. The browser worker
(`src/web/formal_ai_worker.js`) cannot reach Wiktionary or Wikidata
directly because of browser CORS restrictions, so it keeps a small
offline phrase table as a CORS-safe fallback for the GitHub Pages
demo. The fallback returns the same `[<lang>] <surface>` placeholder
the Rust pipeline uses when a lookup misses, so the contract stays
identical across surfaces.

---

## 11. Configuration

All configuration lives in `SolverConfig` and is persisted with the agent
session. The knobs:

| Knob | Type | Default | Effect |
| --- | --- | --- | --- |
| `guess_probability` | f32 in `[0, 1]` | `0.8` | 0 = strongly prefer asking under ambiguity, 1 = always guess. |
| `context_sensitivity` | f32 in `[0, 1]` | `0.6` | how strongly recent messages bias formalization. |
| `questioning_rigor` | f32 in `[0, 1]` | `0.4` | how strict the clarifying question is. |
| `max_decomposition_depth` | usize | `4` | bound on recursive decomposition. |
| `agent_mode` | bool | `false` | unlock destructive / autonomous actions. |
| `diagnostic_mode` | bool | `false` | include trace/intent/evidence chips in the answer prose. |
| `offline` | bool | `false` | refuse external lookups (also `FORMAL_AI_OFFLINE`). |
| `cache_ttl_seconds` | u64 | `5_184_000` | TTL for `source_cache` entries (≈ 60 days). |
| `temperature` | f32 in `[0, 1]` | `0.7` | softmax temperature for interpretation selection. |
| `follow_up_probability` | f32 in `[0, 1]` | see `SolverConfig::default` | how often the proof engine invites the user to refine proof inputs before final execution. |
| `definition_fusion_by_default` | bool | see default | plain definition prompts use cross-language fusion before concept lookup (also `FORMAL_AI_DEFINITION_FUSION`). |
| `associative_project_promotion` | bool | see default | repository questions prefer known Link Assistant / Link Foundation / LinksPlatform projects first. |
| `recursion_mode` | `RecursionMode` | `Down` | which directions of the meta core's recursion are traced (`down`/`up`/`both`); trace-only (R338). |
| `selection_mode` | `SelectionMode` | `Off` | whether the registry-resolved method per atomic leaf is recorded (`off`/`record`); trace-only (R339). |
| `skill_mode` | `SkillMode` | `Off` | whether the skill/curriculum ledger is accumulated (`off`/`accumulate`); proposal-only (R342, also `FORMAL_AI_SKILL_MODE`). |
| `execution_surface` | `ExecutionSurface` | see default | embedding surface used for environment-aware self-description. |
| `blueprint_composition` | `BlueprintComposition` | see default | how composite-program blueprints project their recipe template (issue #340). |
| `probability_policy` | `ProbabilityDecisionPolicy` | paper baseline | `CU`/`TU`/`TC`/`SS` knobs governing how symbolic probability evidence ranks candidates. |
| `forced_response_language` | `Option<&'static str>` | `None` | forces one replay's response language and guards recursion (issue #556, R337-adjacent). |

The same prompt + same config produces the same answer. Random choices are
seeded from the impulse content hash. The defaults live in
`SolverConfig::default` in `src/solver.rs`; the trace-verbosity knobs
(`recursion_mode`, `selection_mode`, `skill_mode`) change neither routing nor
the answer.

---

## 12. Append-Only Event Log

Every event written by the pipeline carries:

- a content-addressed `id` (FNV-1a 64-bit);
- a `kind` naming what was recorded — representative kinds include `impulse`,
  `sub_impulse`, `candidate`, `validation`, `cache_hit`, `source`,
  `source_refresh`, `agent_action`, `trace`, and `error`. The vocabulary is
  open rather than fixed: `EventLog::append` takes any `&'static str`, and many
  kinds use a `prefix:detail` convention (`policy:offline`,
  `formalization:*`, `trace:execution_failure`, `summarization:mode`,
  `meta_algorithm_amendment`, …), which is what the evidence-link namespace
  projects;
- a `payload` that varies by kind (Links Notation snippet).

The in-process `Event` struct (`src/event_log.rs`) is deliberately minimal —
`id`, `kind`, `payload`. Nesting depth, language, and surface are carried by
the payload and the surrounding trace rather than by dedicated fields.

The log is the system of record. The user-facing `answer` field is a
projection. The Links Notation trace is the canonical export form.

`memory::export_full_memory` exports the full bundle (seed + events +
preferences + environment metadata) as one `formal_ai_bundle` Links Notation
file. `memory::import_full_memory` round-trips it back, including known
migrations. Destructive memory maintenance commands reuse the same full-bundle
format for backups before physical deletion or reset.

---

## 13. Surfaces

The same `FormalAiEngine` answers prompts in every surface:

- **Rust library** — `formal_ai::FormalAiEngine::answer` /
  `formal_ai::solve_with_history`.
- **CLI binary** — `formal-ai chat`, `formal-ai memory ...`,
  `formal-ai bundle ...`, operator commands such as
  `formal-ai github-logs ...`, `formal-ai telegram`, `formal-ai serve`.
- **HTTP server** - a local gateway with OpenAI routes under
  `/api/openai/v1`, Anthropic under `/api/anthropic/v1`, Gemini under
  `/api/gemini/v1beta`, Vertex under `/api/vertex/v1`, and native formal-ai
  routes under `/api/formal-ai/v1`. The legacy `/v1/chat/completions`,
  `/v1/responses`, `/v1/messages`, and `/v1/network` aliases remain for existing
  desktop and CLI configs; the older `/v1/graph` alias still resolves but is
  flagged deprecated in favor of `/v1/network`.
- **Desktop shell** — `desktop/main.cjs` starts the same local
  `formal-ai serve` API on loopback, serves the existing `src/web` chat, and
  exposes a preload bridge for API, links-network, full-memory, and permission
  status.
- **Telegram bot** — `POST /telegram/webhook` (webhook) or
  `formal-ai telegram` (long polling).
- **Prepared Telegram Docker image** — releases publish the root `Dockerfile`
  to GitHub Container Registry as `ghcr.io/link-assistant/formal-ai:latest`.
  The repository root `compose.yaml` runs that image with only
  `TELEGRAM_BOT_TOKEN` required, while `FORMAL_AI_DOCKER_IMAGE` lets operators
  point the same compose file at a local build or mirror. The image builds the
  Docker-in-Docker Telegram image: it builds the Rust binary, copies it into
  `konard/box-dind:2.1.1`, keeps
  `/usr/local/bin/dind-entrypoint.sh` as the entrypoint, and defaults to
  `formal-ai telegram --mode polling`. Commands that need nested execution
  use the bundled `$ --isolated docker --auto-remove-docker-container --`
  wrapper from `start-command`, which records logs under
  `/tmp/start-command/logs/`.
- **One-click / one-line services** — the same prepared image runs two managed
  containers: the Telegram bot (`formal-ai-telegram`, the default command) and
  the OpenAI-compatible API server (`formal-ai-server`, `formal-ai serve` for
  agentic mode, published on `127.0.0.1:8080`). The desktop app starts and stops
  both with one click via the testable `desktop/lib/service-control.cjs` module
  (an injected `runDocker` runner, exposed over IPC as
  `formalAiDesktop:serviceStatus` / `startService` / `stopService`); a server
  reproduces the identical containers with `docker compose --profile all up -d`
  (or the documented raw `docker run` arguments). Each service uses its own
  inner-Docker volume (`formal-ai-telegram-docker`, `formal-ai-server-docker`)
  because two DinD daemons cannot share one `/var/lib/docker`. See
  [docs/desktop/service-control.md](docs/desktop/service-control.md).
- **Reasoning projection for client protocols** — the solver's ordered
  `thinking_steps` remain the canonical internal trace. The OpenAI Chat surface
  also emits the rendered trace as `message.reasoning_content` and streaming
  `delta.reasoning_content`; the OpenAI Responses surface emits a
  `type:"reasoning"` output item and `response.reasoning_summary_*` stream
  events; the Anthropic adapter emits `thinking` blocks and `thinking_delta`
  events only for requests that enable extended thinking. This keeps
  thinking-capable CLIs on their native protocol fields without inventing a
  separate display contract.
- **VS Code extension** — `vscode/`, shipped for both hosts: the desktop
  (Node) host drives the local `POST /v1/chat/completions` route, while the
  web/`vscode.dev` host runs the in-process WASM engine. Marketplace and Open
  VSX publication is tracked by issue
  [#666](https://github.com/link-assistant/formal-ai/issues/666).
- **Browser demo** — `src/web/formal_ai_worker.js` (a small loader shim) plus
  the solver logic it `importScripts`-loads from
  `src/web/worker/formal_ai_worker_00.js` … `_21.js`, alongside the WebAssembly
  worker built from `src/web/wasm-worker/src/lib.rs`.

Rust/WASM owns deterministic domain primitives that must match the native
solver byte-for-byte: prompt normalization, language detection, arithmetic
evaluation, stable FNV-1a ids, unknown-answer opener selection, intent-route
matching semantics, web-search provider constants, request evidence, and
reciprocal-rank fusion. JavaScript keeps the browser-only responsibilities: UI
state, seed-file fetch/parsing, network/CORS orchestration, DOM integration,
and compatibility fallbacks when WASM cannot be instantiated.

**The browser boundary is not yet narrow, and this is the honest current
state.** The WASM bridge (`src/web/wasm-worker/src/lib.rs`) is ~500 lines,
while `src/web/worker/*.js` still carries roughly 26,700 lines of solver logic
mirroring the ~90,000-line Rust core — the cross-runtime parity (E34) and
issue #349/#408 handlers were mirrored into JavaScript rather than absorbed
into WASM. Pillar 18 ("Rust-to-WebAssembly parity with JavaScript reserved for
UI/glue") therefore describes the target, not today's split. Absorbing the
remaining worker logic into Rust→WASM — after which the JavaScript surface is
capped and lint-enforced as UI/glue — is tracked by issue
[#658](https://github.com/link-assistant/formal-ai/issues/658) (R380), and is
the blocker for the npm-published engine in issue
[#665](https://github.com/link-assistant/formal-ai/issues/665).

Each surface assembles the same `Context` shape so the pipeline answers
identically. The desktop app intentionally stays a wrapper: it sends prompts
through the local chat-completions route, links network inspection to the native
links-network route, and uses the browser memory import/export path for
`formal_ai_bundle` round-trips.

---

## 14. GitHub Evidence Collection

Issue #115 adds the first concrete operator workflow for turning external
development traces into local, reviewable memory. `src/github_logs.rs` builds
deterministic GitHub CLI capture plans and can execute them into a case-study
directory. `scripts/mine-hive-mind-dataset.rs` wraps that command with the
focused Hive Mind defaults used by the issue #115 case study.

The collector records:

- repository metadata;
- recent issues, pull requests, and workflow runs;
- selected issue bodies and issue comments;
- selected PR bodies, discussion comments, inline review comments, reviews,
  and diffs;
- selected GitHub Actions run metadata and full logs;
- a `manifest.json` that preserves every command used to produce each file.

This is not a reasoning engine by itself and is intentionally not registered
as a seed agent tool. It is the ingestion boundary for real-world traces from
systems such as `link-assistant/hive-mind`, so later solver work can operate
over observed issue text, PR feedback, work-session summaries, CI outcomes,
and run logs instead of undocumented anecdotes.

---

## 15. Testing Architecture

Tests live under `tests/unit/specification/` and follow three patterns:

1. **Active test** — pins a current implementation behavior. Always green on CI.
2. **Tracked requirement test** — `#[ignore = "tracked requirement: ..."]`. Documents a failing
   expectation without blocking CI. Run with `cargo test --include-ignored`.
3. **Matrix test** — `for (prompt, expected) in [..]` table-driven. Used
   for 5–10 input variations per category per language (issue #103). Today
   we get to 5–10 variations per language with zero new dependencies; for
   external-file catalogs the cleanest upgrade is `datatest-stable` + YAML.

The test module split is intentional: each surface or capability gets its
own file under `tests/unit/specification/`, so a contributor adding a new category
adds one file (or extends one matrix) without touching the rest.

---

## 16. Open Questions

The original issue #244 architecture questions, the E1-E20 follow-up batches,
and the reasoning batch E21-E27 are merged (PRs #305-#311). Every message is now
formalized into a Links-Notation intent before routing (`src/intent_formalization.rs`),
unmatched prompts run a reasoning-under-unknowns loop (`src/solver_unknown_reasoning.rs`)
instead of a canned opener, per-language program intents collapse into a parametric
`SelectedRule::WriteProgram`, substitution rules run over link CRUD (`src/substitution.rs`),
natural language can query memory / call APIs / execute code under the permission
model (`src/solver_handlers/`), a bounded isolated agent runs allowlisted commands
(`src/agent.rs`), and a permissive industry benchmark slice is imported
(`data/benchmarks/industry-suite.lino`).

The 2026-05-27 audit (issue #244, fourth pass) found that the largest remaining
gap was the **generality of the synthesis step**. That gap is now closed: the
synthesis batch **E28-E32** ([#313](https://github.com/link-assistant/formal-ai/issues/313)-[#317](https://github.com/link-assistant/formal-ai/issues/317))
is merged (PRs #319-#323). The universal 11-step loop is still the main path for
every prompt (`src/solver.rs::solve_with_history_probability_store_and_intent_cache`),
but the synthesis step (`record_candidates`) now **derives** answers by composing
decomposed sub-results over the links network instead of returning a seed keyed
on the prompt: arithmetic/word-problem and counting answers are computed, Python
functions are synthesized from spec + tests and verified in the bounded agent
workspace (`src/solver_handlers/program_synthesis.rs`), text manipulation is
generalized over arbitrary input, and the imported benchmark suite grew to a
10-case slice that passes **10/10** with a `minimum_pass_count` ratchet
(`tests/unit/specification/benchmarks.rs`).

The 2026-05-29 audit (issue #244, fifth pass) found the next gap is **parity**,
per the PR #245 feedback ("all Rust and JavaScript logic are in sync", "all
languages are supported equally"). The sixth-pass audit (also 2026-05-29) records
that the parity batch is **now closed and merged**:

1. **Universal multilingual operation vocabulary (E33, [#326](https://github.com/link-assistant/formal-ai/issues/326), PR #328).**
   `src/solver_handlers/text_manipulation.rs` no longer triggers on English
   literals. Every operation is recognised by canonicalising the prompt against
   one shared data-driven vocabulary (`data/seed/operation-vocabulary.lino`) that
   lists each operation's surface forms per supported language (`en|ru|hi|zh`),
   mirroring how `intent-routing.lino` already works — general, not per-handler
   literals. Adding a surface form or a whole language is a seed-data edit, not a
   code change. The Rust core loads it via `seed::operation_vocabulary()`; the
   browser worker loads the same file via `src/web/seed_loader.js`.
2. **Cross-runtime parity (E34, [#327](https://github.com/link-assistant/formal-ai/issues/327), PR #329).**
   The JavaScript browser worker (`src/web/formal_ai_worker.js`) now routes
   synthesis prompts through `tryLinkNativeSynthesis`, `tryProgramSynthesis`, and
   `tryTextManipulation`, deriving the same synthesis/numeric/program/text answers
   as the Rust core, verified by the shared fixture
   `data/parity/cross-runtime-synthesis.json`, the Rust test
   `shared_cross_runtime_synthesis_fixture_matches_rust_solver`, and
   `tests/e2e/tests/issue-327.spec.js`. Mirrors the E19 [#282](https://github.com/link-assistant/formal-ai/issues/282)
   browser-worker parity precedent; WebAssembly stays the bridge for shared
   primitives and JavaScript stays UI/glue per pillar 18.

With E1-E34 all merged, no vision-planning epic remains open **for issue #244
specifically**. That statement does not mean planning is finished: two later
batches are open, and `ROADMAP.md` tracks their requirement-level status
(done / partial / not done):

- **E37-E55** ([#656](https://github.com/link-assistant/formal-ai/issues/656)-[#674](https://github.com/link-assistant/formal-ai/issues/674)),
  created from the issue [#651](https://github.com/link-assistant/formal-ai/issues/651)
  gap analysis.
- **E56-E68** ([#698](https://github.com/link-assistant/formal-ai/issues/698)-[#710](https://github.com/link-assistant/formal-ai/issues/710)),
  created from the 2026-07-14 audit of every closed issue and merged PR.

The largest architectural gaps those batches own are: the #559 mandate to
retire the specialized handlers in favour of memory + the meta algorithm
([#663](https://github.com/link-assistant/formal-ai/issues/663),
[#699](https://github.com/link-assistant/formal-ai/issues/699)), real upstream
benchmark execution ([#698](https://github.com/link-assistant/formal-ai/issues/698)),
absorbing the JavaScript worker into WASM
([#658](https://github.com/link-assistant/formal-ai/issues/658)), symbolic
world-model behaviors ([#702](https://github.com/link-assistant/formal-ai/issues/702)),
and driving external agent CLIs as an orchestrator
([#703](https://github.com/link-assistant/formal-ai/issues/703)).

A later issue #349 roadmap closed the concrete program-modification gap that the
parity batch exposed in a user dialog: after an active program artifact exists,
bare follow-ups such as "sort the results in reverse order" are rewritten
against conversation history, decomposed into program modifiers, lowered through
`src/program_plan.rs`, and traced through default-off diagnostics. The #349 flow
is guarded by `tests/integration/issue_349_reverse_sort.rs`, the browser-worker
parity harness `experiments/issue-361-cross-runtime-parity.mjs`, the
coding-modification benchmark ratchet, and the self-improvement specs.

Issue #408 text/code editing path extends that active-artifact behavior to
literal user edits such as "replace Hello World with Bye world". The solver
extracts the prior assistant artifact from conversation history when the prompt
omits an explicit input, applies deterministic text operations from
`src/solver_handlers/text_manipulation.rs`, and mirrors the supported operations
in `src/web/formal_ai_worker.js`. The issue #408 benchmark matrix uses
self-authored benchmark-family examples plus
`data/benchmarks/text-manipulation-suite.lino`, which records 48 researched
sources and drives 30 deterministic local variations per source through a
1,440/1,440 pass-count ratchet. The benchmark gate reports per-source totals:
each source has a 3-check repository-local 10% floor and must pass the stronger
30/30 local ratchet.

Arbitrary natural-language programming beyond the trigger/response subset of
`src/skill_compiler.rs` is closed by E55 (issue #674): `src/skill_procedure.rs`
compiles a freely phrased multi-step procedure by splitting it into ordered
clauses and mapping each onto the step vocabulary seeded in
`data/seed/meanings-skill-procedure.lino` — so growing the vocabulary is a data
edit, not a new Rust match arm. The compiled program carries canonical slugs
only, which is why the same procedure stated in English, Russian, Hindi, or
Chinese content-addresses to one identical set of skill links. A clause with no
vocabulary entry compiles nothing: the solver replies with the named gap and
records a `skill_gap` event, and every compiled step keeps the source sentence
span it was read from so *"why did you do that?"* can quote it.

Pull requests that close any of these should update the corresponding row in
the table in Section 2 and link the new module.

---

## 17. References

- `VISION.md` — values, product story, north-star user experience.
- `GOALS.md` — what counts as success per surface.
- `NON-GOALS.md` — what we explicitly do not build.
- `REQUIREMENTS.md` — issue-by-issue implementation matrix (R1 … R444, plus per-issue blocks such as R499-1…R499-8).
- `ROADMAP.md` — implementation-progress tracker mapping each `VISION.md` pillar to its real code status, closed planning batches, and remaining follow-up gaps.
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs) — default native storage backend.
- [`linksplatform/doublets-web`](https://github.com/linksplatform/doublets-web) — browser-side mirror.
- [`link-assistant/calculator`](https://github.com/link-assistant/calculator) — delegated calculator engine (`link-calculator` crate).
- [`link-assistant/relative-meta-logic`](https://github.com/link-assistant/relative-meta-logic) — future formal-reasoning integration.
- Wikidata (`https://www.wikidata.org/`) — public source of P/Q-ID anchors.
- Wikipedia (`https://*.wikipedia.org/`) — public source of per-language
  concept articles.
- Wiktionary (`https://*.wiktionary.org/`) — public source of per-language
  word and idiom entries.

### Domain background (symbolic AI)

- [Symbolic artificial intelligence](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
  — the field this project belongs to (GOFAI); the source of the best-practice
  audit in `docs/case-studies/issue-451/symbolic-ai-best-practices.md`.
- [Semantic network](https://en.wikipedia.org/wiki/Semantic_network) — the
  classical-AI name for the associative link store.
- [Physical symbol system](https://en.wikipedia.org/wiki/Physical_symbol_system)
  — Newell & Simon's hypothesis underlying the link-as-symbol model.
- [Neuro-symbolic AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI) — the
  integration framing the project tracks while staying pure-symbolic.
- [Boolean satisfiability problem](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
  and the [DPLL algorithm](https://en.wikipedia.org/wiki/DPLL_algorithm) — the
  decision procedure the propositional engine delegates to for claims wider than
  the truth-table limit. The in-house, dependency-free DPLL search lives in
  `src/proof_engine/decision/sat.rs`; wide claims are
  [Tseitin-encoded](https://en.wikipedia.org/wiki/Tseytin_transformation) to CNF
  in `src/proof_engine/decision/boolean.rs` before being handed to it.

### Symbolic world models and contexts (issue #649)

- The design case study in `docs/case-studies/issue-649/README.md` audits how the
  associative stack realizes symbolic **world models**: a **current-state** and a
  **target-state** context, their difference, context **merge/split**, and
  **predicting the consequences of an action** — each context being a **links
  network** rather than an embedding. It maps the request onto the classical
  prior art ([STRIPS/PDDL](https://en.wikipedia.org/wiki/Stanford_Research_Institute_Problem_Solver)
  planning, [truth-maintenance systems](https://en.wikipedia.org/wiki/Reason_maintenance)
  JTMS/ATMS, and [AGM belief revision](https://en.wikipedia.org/wiki/Belief_revision))
  and onto [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic),
  whose kernel already lives in `src/relative_meta_logic.rs`. Statement
  dependency edges and the change-driven recalculation cascade reuse
  `SubstitutionGraph::apply_rules`; the concept-by-concept status is in
  `docs/case-studies/issue-649/world-model-mapping.md`.

### Usage-weighted associative persistence (issue #686)

- `src/associative_persistence.rs` keeps a **persistent** version of
  **meta-language expressions** saved in an **associative links network**: an
  `AssociativeMemory` stores each expression as a content-addressed node (via
  `stable_id`, so one meaning is one node) in an embedded `SubstitutionGraph`,
  counts **usages (reads)** and **changes (writes)** per expression, and derives an
  independent usage signal from each node's **incoming and outgoing link degree**.
  A single `retention_score` (reads + writes + in-degree + out-degree, under
  configurable `RetentionWeights`) drives an LFU-style policy so the **most used,
  most changed, and most connected** knowledge **persists longest**; eviction
  forgets the lowest-scored first. Everything — expressions, read/write counts, and
  associations — is **a link** (never a separate edge/vertex type) and serializes to
  Links Notation. The design case study in
  `docs/case-studies/issue-686/README.md` maps the request onto its prior art
  ([Wikontic](https://huggingface.co/papers/2512.00590) entity-degree↔retrieval,
  [LFU cache replacement](https://en.wikipedia.org/wiki/Cache_replacement_policies),
  [reference counting](https://en.wikipedia.org/wiki/Reference_counting), and
  [degree centrality](https://en.wikipedia.org/wiki/Centrality#Degree_centrality)),
  and the concept-by-concept status is in
  `docs/case-studies/issue-686/persistence-mapping.md`. It generalizes the
  read-count LFU precursor already present in `src/dreaming.rs` (`usage_counts`) and
  bridges to the issue #649 world model via `AssociativeMemory::from_context`.
