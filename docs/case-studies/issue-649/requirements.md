# Issue 649 — Holistic Requirements

Every requirement extracted from the issue body. The issue has **no comments**
([`raw-data/issue-649-comments.json`](raw-data/issue-649-comments.json) is `[]`),
so the body is the complete specification. Requirements are split into the
**conceptual feature** the issue describes (R649-01 … R649-14) and the
**meta-deliverable** it asks this PR to produce (R649-15 … R649-19).

Status legend: **Realized** — already present in the codebase and reused as-is;
**Partial** — a reusable precursor exists but not in the shape the issue wants;
**Proposed** — scoped future work with a concrete plan in
[`solution-plans.md`](solution-plans.md); **Done** — delivered by this PR.

## Conceptual feature requirements

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R649-01** | Use **links networks as the meaning representation**, *instead of embeddings*. | Realized (substrate) — `src/link_store.rs::LinkStore`, `src/links_format.rs::format_lino_record`; the repo is already links-based. |
| **R649-02** | Throughout a dialogue, build and maintain a representation of the **current state of the world** (the partial world discussed). | Partial — `src/memory.rs::MemoryStore` + `src/memory_sync.rs::SyncStore` (append-only dialogue log) and `src/substitution.rs::SubstitutionGraph` (mutable current-state links). |
| **R649-03** | Build and maintain a representation of the **target state of the world** the user wants. | Partial — `src/intent_formalization.rs::IntentFormalization`, `src/change_request.rs::ChangeRequest`, `src/program_plan.rs::ProgramPlan`. |
| **R649-04** | At any dialogue stage, expose the **difference** between the current state and the target state. | Partial — `src/substitution.rs::SubstitutionTraceReport` (before/after link diff), `src/meta_frame.rs::NeedLedger` (satisfied/blocked gap list). |
| **R649-05** | The user can **state what they want** as an edit to the target world model ("explain exactly what he wants"). | Partial — `src/change_request.rs::derive_requirement`, `IntentFormalization`. |
| **R649-06** | On request, the agent can **explain the current state and the target state exactly** (glass-box, not approximate). | Partial — `src/self_explanation.rs`, the append-only event log + `trace:` pointers. |
| **R649-07** | The user can **synchronize** their understanding of the target with the agent **and vice versa**. | Proposed — a bidirectional confirm/amend loop over the target model. |
| **R649-08** | A per-dialogue world (context) model can be **merged** into the entire world model / general context. | Partial — `src/memory_sync.rs::merge_union_by_id` / `merge_event` (union link/event graphs by id). |
| **R649-09** | World models (contexts) can be **split** as needed. | Partial — `src/meta_frame.rs` clause/span splitters + `WorkUnit` decomposition, `src/solver_helpers::record_decomposition`. |
| **R649-10** | **Each context is always a links network.** | Realized (substrate) / Partial (as a named "context") — `SubstitutionGraph`/`LinkStore` are the links network; only a `Context` wrapper (id + subgraph) is missing. |
| **R649-11** | In each world / formal system / context, statements are **dependent** on each other. | Proposed — the relative-meta-logic kernel exists but assesses each statement independently; inter-statement dependency edges are absent. |
| **R649-12** | If something changes in the world model / formal system / context, **all statement probabilities are recalculated**. | Partial (engine) / Proposed (wiring) — `SubstitutionGraph::apply_rules` (fixpoint recompute) and `src/probability.rs::ProbabilityStore::target_weight` (on-demand re-aggregation) exist but are not driven by `TruthValue` cascades. |
| **R649-13** | Use **relative dependent logic (relative meta logic)** — [`link-foundation/relative-meta-logic`](https://github.com/link-foundation/relative-meta-logic) — as the logic layer. | Realized (kernel) — `src/relative_meta_logic.rs` (`TruthValue`, `Aggregator`, `StatementAssessment`). |
| **R649-14** | **Predict the consequences of an action** by simulating it against the world model / formal system / context. | Proposed (thin precursors) — `SubstitutionGraph::apply_rules` (apply action as link edits), Markov transitions in `src/probability.rs`. |

## Meta-deliverable requirements (this PR)

| ID | Requirement | Status |
|---|---|---|
| **R649-15** | **Collect the issue-related data** into `docs/case-studies/issue-649/`. | Done — [`raw-data/`](raw-data/). |
| **R649-16** | Do a **deep case-study analysis**, including **online research** for additional facts. | Done — [`README.md`](README.md) + [`raw-data/online-research.md`](raw-data/online-research.md). |
| **R649-17** | **List each and all requirements** from the issue. | Done — this file (R649-01 … R649-19). |
| **R649-18** | Propose **possible solutions and solution plans for each requirement**, checking **known existing components/libraries**. | Done — [`solution-plans.md`](solution-plans.md). |
| **R649-19** | **Plan and execute everything in the single PR** ([#675](https://github.com/link-assistant/formal-ai/pull/675)). | Done — every artifact in this directory plus `REQUIREMENTS.md` rows R428–R434, the changelog fragment, and the traceability test. |

## Why these nineteen and not more

The issue body is three short paragraphs of intent plus one meta paragraph.
R649-01 is the "links networks instead of embeddings" sentence; R649-02/03 are
the "current state … and the target state" sentence; R649-04/05/06/07 are the
"what is the difference … explain it exactly … synchronize the understanding"
sentences; R649-08/09/10 are the "merge or split world models (contexts) … each
context is always a links network" paragraph; R649-11/12/13 are the
"relative dependent logic … dependent statements … all statements probabilities
are recalculated" paragraph; R649-14 is the issue **title** ("predicting
consequences of actions"). R649-15 … R649-19 are the four sentences of the final
meta paragraph. No requirement is implied beyond these without over-reading the
text.

## Relationship to the global requirement matrix

These per-issue IDs are the fine-grained specification. The
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) global matrix records this PR's
concrete deliverables as rows **R428–R434** under *"Issue #649 World Models And
Contexts"*, each pointing back into this directory.
