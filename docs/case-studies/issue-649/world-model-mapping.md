# World models & contexts, expressed in the associative stack

This document maps **each concept issue #649 names** to the component in
`formal-ai` that realizes it (fully, partially, or not yet), with `path:symbol`
evidence and an honest **Realized / Partial / Proposed** status. It is the
issue-649 analog of
[`issue-451/symbolic-ai-best-practices.md`](../issue-451/symbolic-ai-best-practices.md):
the point was to show the world-model feature is mostly an **audit-and-wire** task
over existing associative machinery, not a green-field subsystem — and PR #675
then **did the wiring**, landing the [`src/world_model.rs`](../../../src/world_model.rs)
module (`Context`, `WorldModel`, `Action`, dependent `Statement`s, `predict`,
`difference`, `merge_from`, `split_off`) that realizes the core of the feature.

Summary: **9 realized (7 of them via the new `world_model` module), 4 partial, 1
proposed** across 14 concept rows. The four **partial** rows are the ones that
still need to be threaded into the live dialogue pipeline (seed the current
context from the append-only log, build the target from `IntentFormalization`,
route "I want …" into target edits, render contexts through `self_explanation`);
the one **proposed** row is the agent⇄user target-sync confirm/amend loop. The
module ships with executable coverage in
[`tests/unit/issue_649_world_model.rs`](../../../tests/unit/issue_649_world_model.rs).

| # | Issue concept | Associative-stack realization | Evidence (`path:symbol`) | Status |
|---|---|---|---|---|
| 1 | Meaning representation = **links network** (not embeddings) | The doublet link store and its Links-Notation serialization | `src/link_store.rs::LinkStore` / `DoubletsLinkStore`; `src/links_format.rs::format_lino_record`; `src/links_query.rs::run_links_query` | Realized (substrate) |
| 2 | **Current-state** world model per dialogue | Append-only dialogue log projected to a links network, plus a mutable current-state link graph | `src/memory.rs::MemoryStore`; `src/memory_sync.rs::SyncStore::record_chat_exchange`; `src/substitution.rs::SubstitutionGraph`; invariant `VISION.md` "appended as events first, then projected into the current state" (`REQUIREMENTS.md` R61) | Partial |
| 3 | **Target-state** world model per dialogue | The desired requirement/task captured at formalization time | `src/intent_formalization.rs::IntentFormalization`; `src/change_request.rs::ChangeRequest::derived_requirement`; `src/program_plan.rs::ProgramPlan::resolved_task` | Partial |
| 4 | **Difference** between current and target | STRIPS goal − state delta (`to_add`/`to_remove`/`conflicting`) over two contexts' link sets | `src/world_model.rs::Context::difference` / `WorldModel::difference`; substrate: `src/substitution.rs::SubstitutionTraceReport`, `src/meta_frame.rs::NeedLedger` | Realized (`world_model`) |
| 5 | User **states what they want** (target edit) | Change-request derivation from an instruction | `src/change_request.rs::derive_requirement`; `src/intent_formalization.rs` | Partial |
| 6 | **Explain current & target exactly** (glass box) | Deterministic self-explanation over the append-only trace | `src/self_explanation.rs`; event log + `trace:` pointers (`src/event_log.rs`) | Partial |
| 7 | **Synchronize** understanding of the target (agent ⇄ user) | — (no confirm/amend loop over a target model yet) | precursor: `src/question_generation.rs` (clarifying questions) | Proposed |
| 8 | **Merge** a context into the general world model | Context combination: union state links + union statements by id (last-writer-wins), then recalculate | `src/world_model.rs::Context::merge_from`, `WorldModel::commit_current_to_general`; substrate: `src/memory_sync.rs::merge_union_by_id` | Realized (`world_model`) |
| 9 | **Split** contexts as needed | Context separation: carve a child context from selected statements + the links referencing them, leaving the parent intact | `src/world_model.rs::Context::split_off`; substrate: `src/meta_frame.rs` clause/span splitters + `WorkUnit` | Realized (`world_model`) |
| 10 | **Each context is a links network** | A named `Context` (id + its own `SubstitutionGraph` + statements) whose statement layer mirrors into the graph and serializes to Links Notation | `src/world_model.rs::Context` (`links_notation`, `assert_link`, `holds`); note: `src/concepts.rs::resolve_context_label` is *linguistic* context, not this world-model container | Realized (`world_model`) |
| 11 | Statements are **dependent** within a context | `Dependency{on, stance}` edges (JTMS positive/negative justifications) between statements; each statement carries its own evidence plus dependency-derived evidence | `src/world_model.rs::{Statement, Dependency}`; reuses `src/relative_meta_logic.rs::StatementAssessment::assess` | Realized (`world_model`) |
| 12 | **Recalculate all probabilities** when the world changes | Bounded-pass JTMS cascade: every context edit re-fires `StatementAssessment` for every statement from the current values of its dependencies, to a fixpoint on the RML decimal grid | `src/world_model.rs::Context::recalculate` (`RecalculationReport`); reuses `src/relative_meta_logic.rs` aggregators | Realized (`world_model`) |
| 13 | Use **relative-meta-logic** as the logic layer | The upstream RML kernel is ported and consumed directly by the recalculation cascade | `src/relative_meta_logic.rs::{TruthValue, Aggregator, RelativeEvidence, StatementAssessment}` | Realized (kernel) |
| 14 | **Predict consequences of an action** | *Predict = apply the action to a clone, recalculate, diff* — never mutates the real model; reports added/removed state links plus every statement whose probability moves | `src/world_model.rs::{Action, Context::predict, Prediction}` | Realized (`world_model`) |

## What PR #675 implemented (was "Proposed")

The audit predicted the world model was an **audit-and-wire** task, and PR #675
did the wiring in [`src/world_model.rs`](../../../src/world_model.rs). The rows
that were *Proposed* are now **Realized** exactly as the plan described:

- **Dependency edges (row 11)** are `Dependency{on, stance}` records on each
  `Statement`; a positive (`Supports`) edge is a JTMS positive justification, a
  negative (`Contradicts`) edge a negative one.
- **Cascade recalculation (row 12)** is `Context::recalculate`: each pass
  re-fires `StatementAssessment::assess` for every statement, feeding it the
  current truth of its dependencies as synthesized full-trust evidence, iterating
  to a fixpoint on the RML decimal grid — the RML query-time semantics (see
  [`raw-data/online-research.md`](raw-data/online-research.md) §2) make this
  correct by construction, and a bounded pass count guarantees termination even
  for a negative-feedback dependency cycle.
- **Consequence prediction (row 14)** is `Context::predict`: clone the context,
  apply the action's add/delete link edits to the clone, recalculate, and diff —
  so the real state is untouched — i.e. rows 4 + 9 + 12 composed.
- **Difference / merge / split (rows 4, 8, 9)** are `Context::difference`,
  `Context::merge_from`, and `Context::split_off`.

The one remaining **Proposed** row is **target sync (row 7)** — the agent⇄user
confirm/amend loop over the target model, which will reuse
`question_generation.rs`. The four **Partial** rows (2, 3, 5, 6) are the
pipeline-integration steps: seed `WorldModel::current` from the append-only log,
build `WorldModel::target` from `IntentFormalization`, route "I want …" into
target edits, and render contexts through `self_explanation` (the module already
exposes `Context::links_notation` for the last).

The [`solution-plans.md`](solution-plans.md) gives the per-requirement plan and
the external prior art each reuse mirrors (STRIPS add/delete effects, ATMS
contexts, JTMS recomputation — see the research file).
