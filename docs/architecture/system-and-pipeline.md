# Architecture: System Context, Pipeline, and Links Notation Input

Part of the [architecture overview](../../ARCHITECTURE.md) (§1–§3). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 1. System Context

The runtime computes an answer from four kinds of input:

1. **Last input message** — the raw text the user just sent.
2. **Previous messages** — the in-process conversation turns, expressed as
   `ConversationTurn { role, text }` (see `rust/src/solver.rs`).
3. **Memory** — the append-only event log (see `rust/src/memory.rs`,
   `rust/src/event_log.rs`) plus the seed dataset under `data/seed/`.
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
| 1. Input | `rust/src/engine.rs::FormalAiEngine::answer` and `solve_with_history` in `rust/src/solver.rs` | Implemented |
| 2. Translate to Links Notation | `EventLog::append("impulse", …)` in `rust/src/event_log.rs` | Implemented |
| 3. Record in memory | `MemoryStore::append` in `rust/src/memory.rs` | Implemented |
| 4. Formalization | `rust/src/concepts.rs` plus `rust/src/translation/formalization.rs` for scored P/Q-id, Wikipedia, Wiktionary, and raw fallback anchors | Implemented for surface-form anchoring and, since issue #1138 B4, concept-graph formalization whose unresolved surfaces become explicit needs |
| 4b. Coding-discovery grounding | `rust/src/coding/concept_discovery.rs::discover_with_lookup` over `rust/src/concept_lookup.rs` and the trusted sources in `data/seed/sources-registry.lino` | Implemented (issue #1138 B1/B4): needs are grounded by live lookup when the run is online, reported unresolved with their source span otherwise |
| 5. Temperature interpretation selection | `rust/src/translation/selection.rs`, `rust/src/probability.rs`, plus `SolverConfig::{temperature, guess_probability, questioning_rigor}` in `rust/src/solver.rs` | Implemented |
| 6. Universal solver | `UniversalSolver` in `rust/src/solver.rs` | Implemented |
| 7. Append to memory | `event_log::EventLog`, `memory::export_full_memory` | Implemented |
| 8. Render user-facing answer | `SymbolicAnswer` projection in `rust/src/engine.rs` | Implemented |
| 9. Natural-language skill compilation | `rust/src/skill_compiler.rs` plus the `behavior_rules` replay bridge, and `rust/src/skill_procedure.rs` plus `rust/src/solver_handlers/procedure_rules.rs` for freely phrased procedures | Implemented for deterministic trigger/response skill packages and for multi-step procedures stated in ordinary prose |

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
`rust/src/solver_dispatch.rs` keeps only
the executable function pointers (`HANDLER_FUNCTIONS`, which must stay Rust) and
`specialized_handlers()` joins the two — asserting at load time that the seed is
an exact permutation of the registry, so a seed edit can never silently drop or
duplicate a handler. Reordering two rows in the seed changes routing (proven by
`cargo test routing_precedence_from_seed`); the shipped seed preserves today's
behaviour. `MethodRegistry::from_dispatch` (`rust/src/method_registry.rs`) surfaces
this seed-ordered table as the `Specialized` method surface, and
`meta_method_dispatch::try_dispatch` consumes it. The browser worker mirrors the
seed through `js/seed_loader.js`; because it names its handlers differently
and runs its async fetch handlers in a later phase, full order-parity is
impossible, so `rust/tests/fixtures/routing-parity.lino` pins the *shared* precedence
invariants both surfaces must honour (checked by the routing-parity test).

The precedence is also something Formal AI re-derives *itself*, through its own
Agent CLI: the rationale behind the ordering (`#395`, `#423`, `#425`, `#552`,
http_fetch-first, incompatible_units-last) is a persisted associative links
network (`data/meta/issue-663-handler-precedence-learning.lino`) that the
`handler_precedence_learning` report ranks into a human-review-gated proposal.
The report is one row in the `REPORTS` table
(`rust/src/agentic_coding/learning_report.rs`) — data-routed, not a planner branch —
and its committed evidence is byte-for-byte reproducible by the in-process
renderer (`rust/tests/unit/issue_663_handler_precedence_learning.rs`), so the tool,
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
