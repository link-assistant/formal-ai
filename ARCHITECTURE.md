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
|  rank candidates by score; apply temperature softmax       |
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
| 2. Translate to Links Notation | `event_log::Event::Impulse` in `src/event_log.rs` | Implemented |
| 3. Record in memory | `MemoryStore::append` in `src/memory.rs` | Implemented |
| 4. Formalization | `concepts::lookup_concept` in `src/concepts.rs` (today: aliases). Future: P/Q-ID extraction with wikidata cache | Partial (alias resolution today; P/Q-ID extraction tracked as a requirement) |
| 5. Temperature interpretation selection | `SolverConfig::guess_probability` already lives in `src/solver.rs`; the scoring/softmax helper is the next implementation step | Knob present; softmax pending |
| 6. Universal solver | `UniversalSolver` in `src/solver.rs` | Implemented |
| 7. Append to memory | `event_log::EventLog`, `memory::export_full_memory` | Implemented |
| 8. Render user-facing answer | `SymbolicAnswer` projection in `src/engine.rs` | Implemented |

The pipeline runs the same way for every prompt — greetings, identity,
concept lookup, math, code generation, idioms, refusals, agent actions —
because the universal solver is intentionally domain-agnostic. Specialized
handlers (`solver_handler_units`, `solver_handler_how`, `solver_handlers`,
`solver_handlers_policy`) are *plugged into* the universal solver, not
branched on by domain at the top level.

---

## 3. Translating Input to Links Notation

The chat surface stores the raw impulse as a Links Notation node first, then
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

Implemented today by `src/event_log.rs` and `src/memory.rs`. Every event is
content-addressed (FNV-1a 64-bit) and appended. The log is exposed through
`SymbolicAnswer::evidence_links` (short, user-visible) and
`SymbolicAnswer::links_notation` (full trace).

### 4.2 Durable doublets-rs / doublets-web store

This is the long-term direction (see `VISION.md` "Current Direction"). The
in-process log will be projected into the doublet links store on a regular
cadence; the doublet store will then be the single physical representation
of the network. Browser storage (`localStorage`, IndexedDB) holds a mirror
for offline-first chat; the disk-side backup writes `.lino` snapshots in the
canonical Links Notation format.

Upstream references:

- [`link-foundation/doublets-rs`](https://github.com/link-foundation/doublets-rs)
- [`link-foundation/doublets-web`](https://github.com/link-foundation/doublets-web)

Migration plan:

1. Wrap the current `MemoryStore` in a trait so the active backend is
   swappable.
2. Add a `doublets-rs` backend behind a feature flag.
3. Mirror writes to `.lino` snapshots via `memory::export_links_notation`.
4. Replace the per-surface tables with the unified doublets store.
5. Add a `doublets-web` backend for the browser worker.

### 4.3 Public-knowledge cache

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

The current implementation implements alias-based formalization in
`src/concepts.rs` (the `aliases` field on every concept record). The full
P/Q-id pipeline is scheduled in `tests/unit/specification/multilingual.rs` under
`russian_iir_evidence_includes_wikidata_anchor` (already active for the IIR
case study); broader coverage is tracked as a requirement.

---

## 6. Temperature-Based Interpretation Selection

Each candidate formalization carries a `score` field. The solver normalizes
the scores by a softmax controlled by the same temperature knob a neural
network would use:

```text
P(c_i) = exp(score_i / T) / Σ exp(score_j / T)
```

- `T → 0`  → deterministic; the highest-scored candidate always wins.
- `T = 1`  → proportional to the raw scores.
- `T → ∞`  → uniform; any candidate is equally likely.

The temperature is sourced from `SolverConfig`. Today we expose a
deterministic default; the softmax helper and a `solver_config.temperature`
field are the next slice of implementation work.

If the top-two probabilities are within ε (configurable through
`SolverConfig.questioning_rigor`), the solver:

- if `guess_probability` is high, picks the higher-scored candidate and
  records a `policy:guessed_under_ambiguity` event so the trace is
  honest;
- otherwise, emits a clarifying-question intent (the smallest question that
  separates the candidates) and stops the pipeline until the user replies.

The seeded-from-impulse-hash RNG (`src/solver.rs::Rng`) keeps the random
guessing deterministic per prompt, so the same input + same config produces
the same answer.

---

## 7. Universal Problem Solver

The solver follows the universal loop documented in `VISION.md` (Section
"Universal Problem-Solving Algorithm"). The implementation is in
`src/solver.rs`:

1. **Impulse** — `Event::Impulse` is appended.
2. **Formalization** — alias resolution + (future) P/Q-id lookup.
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

The associative store supports four kinds of rules. They are listed in
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

The compilation chain (NL → code → binary) is the long-term path. The
runtime never *requires* compilation: a natural-language skill can be
interpreted one step at a time without ever being lowered to Rust/JS.

---

## 10. Translation Between Languages

Because formalization is language-independent (a Wikidata Q-id is the same
whether it is named in English, Russian, Hindi, or Chinese), translation is
not a separate model — it is a re-rendering of the same formalized graph
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
to a formalized graph the same way and re-renders into any other language
the renderer supports. The renderer is a transformation rule (Section 9):
the input is `(graph, target_language)`; the output is rendered text.

---

## 11. Configuration

All configuration lives in `SolverConfig` and is persisted with the agent
session. The knobs:

| Knob | Type | Default | Effect |
| --- | --- | --- | --- |
| `guess_probability` | f32 in `[0, 1]` | `0.5` | 0 = always ask a clarifying question, 1 = always guess. |
| `context_sensitivity` | f32 in `[0, 1]` | `0.7` | how strongly recent messages bias formalization. |
| `questioning_rigor` | f32 in `[0, 1]` | `0.5` | how strict the clarifying question is. |
| `max_decomposition_depth` | usize | `6` | bound on recursive decomposition. |
| `agent_mode` | bool | `false` | unlock destructive / autonomous actions. |
| `diagnostic_mode` | bool | `false` | include trace/intent/evidence chips in the answer prose. |
| `offline` | bool | `false` | refuse external lookups (also `FORMAL_AI_OFFLINE`). |
| `cache_ttl_seconds` | u64 | `5_184_000` | TTL for `source_cache` entries (≈ 60 days). |
| `temperature` *(planned)* | f32 | `1.0` | softmax temperature for interpretation selection. |

The same prompt + same config produces the same answer. Random choices are
seeded from the impulse content hash.

---

## 12. Append-Only Event Log

Every event written by the pipeline carries:

- a content-addressed `id` (FNV-1a 64-bit);
- a `kind` from a fixed vocabulary (`impulse`, `language_detected`,
  `local_search`, `external_search`, `sub_impulse`, `candidate`, `test`,
  `validation`, `policy`, `agent_action`, `cache_hit`, `source`, `trace`,
  `error`, `simplification`, `source_refresh`);
- the `parent_id` it belongs to (so nested traces preserve depth);
- the original `language` and `surface`;
- a `payload` that varies by kind (Links Notation snippet).

The log is the system of record. The user-facing `answer` field is a
projection. The Links Notation trace is the canonical export form.

`memory::export_full_memory` exports the full bundle (seed + events +
preferences + environment metadata) as one `formal_ai_bundle` Links Notation
file. `memory::import_full_memory` round-trips it back, including known
migrations.

---

## 13. Surfaces

The same `FormalAiEngine` answers prompts in every surface:

- **Rust library** — `formal_ai::FormalAiEngine::answer` /
  `formal_ai::solve_with_history`.
- **CLI binary** — `formal-ai chat`, `formal-ai memory ...`,
  `formal-ai bundle ...`, `formal-ai telegram`, `formal-ai serve`.
- **HTTP server** — `POST /v1/chat/completions`, `POST /v1/responses`,
  `GET /health`, `GET /v1/graph` (with `?trace=` filter and
  `?format=dot`).
- **Telegram bot** — `POST /telegram/webhook` (webhook) or
  `formal-ai telegram` (long polling).
- **Browser demo** — `src/web/formal_ai_worker.js` plus the WebAssembly
  worker built from `src/web/wasm-worker/src/lib.rs`.

Each surface assembles the same `Context` shape so the pipeline answers
identically.

---

## 14. Testing Architecture

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

## 15. Open Questions

These items are tracked as requirements today and as architecture
references here:

1. The full Wikidata P-ID / Q-ID formalization (Section 5) is partially
   implemented in `src/concepts.rs` (aliases). Full extraction over arbitrary
   prompts needs a wikidata cache, a multilingual labels table, and a
   per-language morphology hint.
2. The softmax temperature helper (Section 6) is not yet exposed; the knob
   lives on `SolverConfig` but the softmax + ε-comparison helper is the next
   slice of work.
3. The doublets-rs backend (Section 4.2) is wrapped behind a trait but the
   crate dependency is not yet pulled in.
4. Natural-language-skill compilation (Section 9 #5) is documented but the
   compiler is not implemented; today every skill is interpreted by the
   universal solver step by step.

Pull requests that close any of these should update the corresponding row in
the table in Section 2 and link the new module.

---

## 16. References

- `VISION.md` — values, product story, north-star user experience.
- `GOALS.md` — what counts as success per surface.
- `NON-GOALS.md` — what we explicitly do not build.
- `REQUIREMENTS.md` — issue-by-issue implementation matrix (R1 … R136).
- [`link-foundation/doublets-rs`](https://github.com/link-foundation/doublets-rs) — long-term storage backend.
- [`link-foundation/doublets-web`](https://github.com/link-foundation/doublets-web) — browser-side mirror.
- [`link-assistant/calculator`](https://github.com/link-assistant/calculator) — delegated calculator engine (`link-calculator` crate).
- [`link-assistant/relative-meta-logic`](https://github.com/link-assistant/relative-meta-logic) — future formal-reasoning integration.
- Wikidata (`https://www.wikidata.org/`) — public source of P/Q-ID anchors.
- Wikipedia (`https://*.wikipedia.org/`) — public source of per-language
  concept articles.
- Wiktionary (`https://*.wiktionary.org/`) — public source of per-language
  word and idiom entries.
