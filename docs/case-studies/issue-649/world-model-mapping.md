# World models & contexts, expressed in the associative stack

This document maps **each concept issue #649 names** to the component in
`formal-ai` that realizes it (fully, partially, or not yet), with `path:symbol`
evidence and an honest **Realized / Partial / Proposed** status. It is the
issue-649 analog of
[`issue-451/symbolic-ai-best-practices.md`](../issue-451/symbolic-ai-best-practices.md):
the point is to show the world-model feature is mostly an **audit-and-wire** task
over existing associative machinery, not a green-field subsystem.

Summary: **3 realized (substrate), 8 partial, 3 proposed** across 14 concept rows.

| # | Issue concept | Associative-stack realization | Evidence (`path:symbol`) | Status |
|---|---|---|---|---|
| 1 | Meaning representation = **links network** (not embeddings) | The doublet link store and its Links-Notation serialization | `src/link_store.rs::LinkStore` / `DoubletsLinkStore`; `src/links_format.rs::format_lino_record`; `src/links_query.rs::run_links_query` | Realized (substrate) |
| 2 | **Current-state** world model per dialogue | Append-only dialogue log projected to a links network, plus a mutable current-state link graph | `src/memory.rs::MemoryStore`; `src/memory_sync.rs::SyncStore::record_chat_exchange`; `src/substitution.rs::SubstitutionGraph`; invariant `VISION.md` "appended as events first, then projected into the current state" (`REQUIREMENTS.md` R61) | Partial |
| 3 | **Target-state** world model per dialogue | The desired requirement/task captured at formalization time | `src/intent_formalization.rs::IntentFormalization`; `src/change_request.rs::ChangeRequest::derived_requirement`; `src/program_plan.rs::ProgramPlan::resolved_task` | Partial |
| 4 | **Difference** between current and target | Before/after link-set diff from graph rewriting; a per-need satisfied/blocked ledger | `src/substitution.rs::SubstitutionTraceReport`; `src/meta_frame.rs::NeedLedger` (`Satisfied`/`Blocked`, `every_need_accounted_for`) | Partial |
| 5 | User **states what they want** (target edit) | Change-request derivation from an instruction | `src/change_request.rs::derive_requirement`; `src/intent_formalization.rs` | Partial |
| 6 | **Explain current & target exactly** (glass box) | Deterministic self-explanation over the append-only trace | `src/self_explanation.rs`; event log + `trace:` pointers (`src/event_log.rs`) | Partial |
| 7 | **Synchronize** understanding of the target (agent ⇄ user) | — (no confirm/amend loop over a target model yet) | precursor: `src/question_generation.rs` (clarifying questions) | Proposed |
| 8 | **Merge** a context into the general world model | Union of memory/link graphs by id, last-writer-wins per field | `src/memory_sync.rs::merge_union_by_id` / `merge_event` | Partial |
| 9 | **Split** contexts as needed | Prompt/clause span splitting and recursive work-unit decomposition | `src/meta_frame.rs` clause/span splitters + `WorkUnit`; `src/solver_helpers::record_decomposition` | Partial |
| 10 | **Each context is a links network** | The links network exists; a named `Context` (id + its own subgraph) wrapper does not | substrate: `src/substitution.rs::SubstitutionGraph`, `src/link_store.rs::LinkStore`; note: `src/concepts.rs::resolve_context_label` is *linguistic* context, not a world-model container | Realized (substrate) / Partial (as "context") |
| 11 | Statements are **dependent** within a context | — (`StatementAssessment::assess` reads one statement's own evidence list; no inter-statement dependency graph) | `src/relative_meta_logic.rs::StatementAssessment::assess` (independent per statement) | Proposed |
| 12 | **Recalculate all probabilities** when the world changes | Fixpoint graph recompute + on-demand evidence re-aggregation exist, but are not wired to `TruthValue` cascades | engine: `src/substitution.rs::SubstitutionGraph::apply_rules` (fixpoint); `src/probability.rs::ProbabilityStore::target_weight`, `reinforce_transition_path` | Partial (engine) / Proposed (wiring) |
| 13 | Use **relative-meta-logic** as the logic layer | The upstream RML kernel is ported | `src/relative_meta_logic.rs::{TruthValue, Aggregator, RelativeEvidence, StatementAssessment}` | Realized (kernel) |
| 14 | **Predict consequences of an action** | Apply an action as link edits and read the resulting graph; Markov next-state weight | `src/substitution.rs::SubstitutionGraph::apply_rules`; `src/probability.rs` `ProbabilityModel::MarkovTransition`, `applies_to_markov_state` | Proposed (thin precursors) |

## What "Proposed" concretely means here

The three **Proposed** rows (7, 11, 14) and the wiring half of row 12 are not
missing infrastructure — they are missing **connections** between components that
already exist:

- **Dependency edges (row 11)** are just links in a `SubstitutionGraph` whose
  `from`/`to` are statement ids; adding them needs no new store.
- **Cascade recalculation (row 12)** is what `apply_rules` already does for links;
  the work is to represent each statement's `TruthValue` as graph state and let
  the fixpoint drive `StatementAssessment` re-evaluation — the RML query-time
  semantics (see [`raw-data/online-research.md`](raw-data/online-research.md) §2)
  make this correct by construction.
- **Consequence prediction (row 14)** is `apply_rules` run on a *copied* context
  (so the real state is untouched) plus a diff of the result — i.e. rows 4 + 9 +
  12 composed.
- **Target sync (row 7)** reuses `question_generation.rs` to confirm/amend the
  target model.

The [`solution-plans.md`](solution-plans.md) gives the per-requirement plan and
the external prior art each reuse mirrors (STRIPS add/delete effects, ATMS
contexts, JTMS recomputation — see the research file).
