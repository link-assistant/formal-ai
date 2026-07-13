# Issue 649 Case Study

> **Status:** Data collected, requirements enumerated, solution plans drafted, **and the core world-model feature implemented** in PR #675.
> **Type:** Research + design case study **plus implementation** — the audit found the feature was an audit-and-wire task, and PR #675 then landed the [`src/world_model.rs`](../../../src/world_model.rs) module that wires it up.
> **Primary source:** the issue body (three intent paragraphs + one meta paragraph); the PR carries the maintainer's follow-up "*Now we need to fully implement it here in this pull request.*"

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/649>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/675>
- **Concept → stack mapping:** [`world-model-mapping.md`](world-model-mapping.md)
- **Per-requirement plans + prior art:** [`solution-plans.md`](solution-plans.md)
- **Requirement list:** [`requirements.md`](requirements.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

Issue #649 — *"Predicting consequences of actions using world models / formal
systems / contexts"* — asks the project to reason with **symbolic world models**:
throughout a dialogue build a **current-state** model and a **target-state**
model, expose their **difference**, let the user and agent **synchronize** the
target, **merge** and **split** context models, keep **each context a links
network** (explicitly *not* embeddings), use
[**relative-meta-logic**](https://github.com/link-foundation/relative-meta-logic)
so that **statements are dependent** and **changing the world recalculates all
statement probabilities**, and ultimately **predict the consequences of an
action** by simulating it. The issue was inspired by a world-model video —
identified as *"Yann LeCun's $1B Bet Against LLMs [Part 1]"* by Welch Labs (see
[`raw-data/online-research.md`](raw-data/online-research.md) §1) — but swaps
LeCun's learned embeddings for a links network.

The final meta paragraph is the concrete deliverable: **collect the issue data**
into `docs/case-studies/issue-649/`, do a **deep case-study analysis** with
**online research**, **list every requirement**, **propose solution plans** per
requirement while **surveying existing components/libraries**, and **do it all in
this single PR**.

The central finding was that **the associative stack already provides every
substrate the feature needs** — a links network (`LinkStore`), a fixpoint rewrite
engine (`SubstitutionGraph::apply_rules`), the relative-meta-logic kernel
(`src/relative_meta_logic.rs`), symbolic probability (`src/probability.rs`), and
context merge (`memory_sync::merge_union_by_id`). What was missing was not
infrastructure but **connections**: a named `Context` wrapper, inter-statement
**dependency edges**, a `TruthValue` **cascade** on change, and a **predict =
apply-to-a-copy + diff** composition. PR #675 supplied exactly those connections
in the [`src/world_model.rs`](../../../src/world_model.rs) module, so the honest
status is now **9 realized concept rows (7 of them via the new module), 4 partial,
1 proposed** (see [`world-model-mapping.md`](world-model-mapping.md)). The four
partial rows are pipeline-integration steps (seed the current context from the
append-only log, build the target from `IntentFormalization`, route "I want …"
into target edits, render contexts through `self_explanation`); the one proposed
row is the agent⇄user target-sync loop.

The deliverables are code + documentation, fully traceable:

1. **The implementation:** [`src/world_model.rs`](../../../src/world_model.rs) —
   `Context` (a links network + dependent statements), `WorldModel`
   (`current`/`target`/`general`), `Action` (STRIPS-style link edits),
   `Context::{difference, predict, recalculate, merge_from, split_off}`, and the
   RML-backed dependent-statement cascade — with executable coverage in
   [`tests/unit/issue_649_world_model.rs`](../../../tests/unit/issue_649_world_model.rs).
2. This case study, the requirement list, the concept→stack mapping, and the
   solution plans, all under `docs/case-studies/issue-649/`.
3. `REQUIREMENTS.md` rows **R428–R434** under *"Issue #649 World Models And
   Contexts"*.
4. `tests/unit/docs_requirements_issue_649.rs` pins the reference, the headings,
   and the requirement IDs so the case study cannot silently regress.

---

## 2. Collected Data

The raw, third-party captures (exempt from authored-prose lints) are archived
under [`raw-data/`](raw-data/):

| File | What it is |
|---|---|
| [`raw-data/issue-649.json`](raw-data/issue-649.json) | The issue as filed (`gh issue view 649 --json …`). |
| [`raw-data/issue-649-comments.json`](raw-data/issue-649-comments.json) | Issue comment thread — empty (`[]`); the body is the sole specification. |
| [`raw-data/pr-675.json`](raw-data/pr-675.json) | The draft pull request this work lands in. |
| [`raw-data/pr-675-conversation-comments.json`](raw-data/pr-675-conversation-comments.json) | PR conversation comments — empty (`[]`). |
| [`raw-data/pr-675-review-comments.json`](raw-data/pr-675-review-comments.json) | PR inline review comments — empty (`[]`). |
| [`raw-data/pr-675-reviews.json`](raw-data/pr-675-reviews.json) | PR reviews — empty (`[]`). |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research: the source video, relative-meta-logic, STRIPS/PDDL classical planning, situation calculus, truth-maintenance systems (JTMS/ATMS), AGM belief revision, and the modern world-model literature. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), the research file
quotes only short definitional phrases and links every claim to its source.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body is enumerated in
[`requirements.md`](requirements.md) as **R649-01 … R649-19** (the conceptual
feature plus the meta-deliverable). These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R428–R434** under *"Issue #649
World Models And Contexts"*. The short form:

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R428** | Collect the issue-649 data into a dedicated case-study directory. | Done — [`raw-data/`](raw-data/). |
| **R429** | Deep case-study analysis with cited online research. | Done — this file + [`raw-data/online-research.md`](raw-data/online-research.md). |
| **R430** | Enumerate every requirement of the issue. | Done — [`requirements.md`](requirements.md) (R649-01 … R649-19). |
| **R431** | Map each world-model concept to the associative stack with honest status. | Done — [`world-model-mapping.md`](world-model-mapping.md) (3 realized / 8 partial / 3 proposed). |
| **R432** | Propose a solution plan per requirement, surveying existing components/libraries. | Done — [`solution-plans.md`](solution-plans.md). |
| **R433** | Plan and execute everything in the single PR #675. | Done — every artifact here plus the changelog fragment and traceability test. |
| **R434** | Protect the case study with a documentation-traceability regression test. | Done — `tests/unit/docs_requirements_issue_649.rs`. |

See [`requirements.md`](requirements.md) §"Why these nineteen" for why the issue
decomposes into exactly these requirements and no more.

---

## 4. Deep Analysis — world models on the associative stack

### 4.1 The issue re-expresses classical planning in links, not embeddings

The video the issue cites argues for *world models that predict the consequences
of actions* — LeCun's JEPA/V-JEPA line learns those models as **embeddings**. The
issue keeps the **goal** (predict consequences, plan toward a target) but
**rejects the representation**: every context must be a **links network**. That
is precisely the **STRIPS/PDDL** worldview — a *state* (set of true atoms), a
*goal*, and *actions* with add/delete effects — re-expressed over doublets. The
current-state model is STRIPS' initial state; the target-state model is its goal;
an action is a set of link edits; the *difference* is the goal−state delta an
action must achieve; *predicting a consequence* is computing the successor state.
The associative stack already speaks this language: a context is a
`SubstitutionGraph`, and applying an action is `apply_rules`.

### 4.2 Dependent statements = a truth-maintenance system

The issue's *"statements are dependent … if something changes, all statement
probabilities are recalculated"* is the definition of a **truth-maintenance
system**. A JTMS records **justifications** between beliefs and re-evaluates the
affected ones when an assumption changes; an ATMS additionally maintains
**multiple contexts** at once — exactly the merge/split the issue wants. The
present `StatementAssessment::assess` (verified at
`src/relative_meta_logic.rs:351`) computes each statement's `TruthValue` from
**its own** evidence list independently — a correct RML per-statement kernel, but
with **no inter-statement dependency graph**. The gap is therefore narrow and
named: represent dependencies as links, and let the existing `apply_rules`
fixpoint drive re-assessment (JTMS-style). Relative-meta-logic's query-time
re-evaluation semantics make the cascade correct by construction (research §2).

### 4.3 Merge / split / synchronize = ATMS contexts + AGM revision

Merging a per-dialogue context into the general world model is ATMS
context-combination; splitting is maintaining separate contexts; synchronizing
the target with the user is **AGM belief revision** — the rational update of a
belief set under new input. The stack already has the mechanical halves:
`memory_sync::merge_union_by_id` (union by id, last-writer-wins) for merge, the
`meta_frame` clause/span splitters and `WorkUnit` decomposition for split, and
`question_generation.rs` for the confirm/amend loop. What is missing is scoping
them to a first-class `Context` object — a wrapper, not a new store.

### 4.4 Plan first, then implement

The issue's meta paragraph asks to *"propose possible solutions and solution
plans for each requirement"* and to *"check known existing components/libraries"*
— so this case study did the audit first: name what already realizes each
requirement and name the concrete reuse target for each gap. The maintainer then
asked on PR #675 to *"fully implement it here in this pull request"*, and the
audit's payoff is that the implementation is a thin wiring layer, not a
green-field engine: [`src/world_model.rs`](../../../src/world_model.rs) reuses
`SubstitutionGraph` as the state container, `StatementAssessment::assess` as the
per-statement kernel, and `stable_id` for content addressing, adding only the
`Context`/`WorldModel`/`Action` types and the dependency cascade the audit
identified as missing. Two guard-rails from the audit are honored in code: the
recalculation cascade is **bounded** (it cannot loop on a negative-feedback
dependency cycle) and `predict` runs on a **clone** (it never introduces a second
mutable source of truth for the current state, preserving the append-only
invariant of `VISION.md` / `REQUIREMENTS.md` R61).

### 4.5 What the modern literature adds

The world-model literature (JEPA, V-JEPA 2, energy-based planning; research §6)
converges on *predict-then-plan*, but over learned latent states. `formal-ai`'s
bet is that the *predict-then-plan* half can stand on a **readable links network**
for the task classes it targets — the same wager the project makes elsewhere
(issue-451 §4.5). Nothing in the literature contradicts a symbolic world model;
it simply optimizes a different axis (perceptual generality) than the one this
project optimizes (inspectability and determinism).

---

## 5. Concept → Associative Stack (overview)

The full mapping with `path:symbol` evidence and status is in
[`world-model-mapping.md`](world-model-mapping.md). Summary — **9 realized (7 via
the new `world_model` module), 4 partial, 1 proposed** across 14 concept rows:

| Issue concept | Associative-stack realization | Status |
|---|---|---|
| Links network as meaning (not embeddings) | `LinkStore` / Links-Notation | Realized |
| Current-state model | `WorldModel::current`; log-seeding not yet wired | Partial |
| Target-state model | `WorldModel::target`; `IntentFormalization` build not yet wired | Partial |
| Current↔target difference | `world_model::Context::difference` | Realized |
| User states the target | `change_request::derive_requirement` routing not yet wired | Partial |
| Explain current & target exactly | `Context::links_notation`; `self_explanation` wiring pending | Partial |
| Synchronize the target | `question_generation` (precursor) | Proposed |
| Merge contexts | `world_model::Context::merge_from` | Realized |
| Split contexts | `world_model::Context::split_off` | Realized |
| Each context is a links network | `world_model::Context` (id + `SubstitutionGraph` + statements) | Realized |
| Dependent statements | `world_model::{Statement, Dependency}` (JTMS justifications) | Realized |
| Recalculate all probabilities on change | `world_model::Context::recalculate` (bounded RML cascade) | Realized |
| Use relative-meta-logic | `src/relative_meta_logic.rs` | Realized |
| Predict consequences of an action | `world_model::Context::predict` (apply-to-clone + diff) | Realized |

---

## 6. Solution Plans

The per-requirement solution plan and the existing-component reuse target for
each requirement are in [`solution-plans.md`](solution-plans.md), which also
carries the full **prior-art survey**. The one-paragraph architecture — now
implemented in [`src/world_model.rs`](../../../src/world_model.rs) — is a
first-class `Context = { id, SubstitutionGraph, statements }`, a `WorldModel`
holding `current` + `target` + `general` contexts, an action modeled as link
edits (STRIPS effects), consequence prediction as apply-the-action to a **clone**
followed by a diff, and recalculation as a bounded fixpoint that re-fires
`StatementAssessment` for dependent statements (JTMS-style).

---

## 7. Existing Components / Prior Art Surveyed

Detailed in [`solution-plans.md`](solution-plans.md); summary:

- **In-repo (reused, not re-bought):** `SubstitutionGraph` (state + recompute),
  `relative_meta_logic` (RML kernel), `probability` (Bayes/Markov evidence),
  `memory_sync` (merge), `meta_frame` (split + gap ledger), `link_store` /
  `links_format` (the links network).
- **External formalisms re-expressed:** STRIPS/PDDL (state + goal + add/delete
  effects), situation calculus, JTMS/ATMS truth maintenance, AGM belief revision,
  upstream **relative-meta-logic**.
- **External systems surveyed, intentionally not adopted:** JEPA / V-JEPA 2 /
  energy-based world models — out of scope by construction because the issue
  requires links networks *instead of* embeddings.

**Net conclusion:** for every requirement, either an in-repo component already
realizes it (and is cited) or a specific, named component is the concrete reuse
target for a *Proposed* wiring step — no requirement is left both unrealized and
unplanned.

---

## 8. Risks

| Risk | Why it matters here | Mitigation in the plan |
|---|---|---|
| **Second source of truth** | A standalone current-state store could drift from the append-only log. | Project the current context *from* the log (`VISION.md`/R61); never write it independently. |
| **Cascade non-termination** | A `TruthValue` cascade over dependency edges could loop (negative-feedback cycle). | Implemented: `Context::recalculate` runs a **bounded** relaxation (`MAX_RECALCULATION_PASSES_PER_STATEMENT × statement count`) and snaps values to the RML decimal grid; a regression test drives a mutual-contradiction cycle to prove termination. |
| **Combinatorial explosion of consequences** | Simulating every action against a large context is expensive. | Predict on a *scoped copy* of the relevant subgraph (split first), not the whole world model. |
| **Overclaiming completeness** | Marketing a half-built engine would violate `NON-GOALS.md`. | Honest Realized/Partial/Proposed status per row; proposed rows carry concrete reuse targets, not vague intent. |
| **Terminology collision** | `concepts.rs::resolve_context_label` already means *linguistic* context. | The world-model `Context` is a distinct type; the mapping notes the distinction explicitly. |

---

## 9. Files

```
docs/case-studies/issue-649/
├── README.md                 # this analysis
├── requirements.md           # R649-01 … R649-19 (R430)
├── world-model-mapping.md    # concept → associative-stack mapping (R431)
├── solution-plans.md         # per-requirement plans + prior-art survey (R432)
└── raw-data/                 # third-party captures (lint-exempt)
    ├── issue-649.json
    ├── issue-649-comments.json          # empty
    ├── pr-675.json
    ├── pr-675-conversation-comments.json # empty
    ├── pr-675-review-comments.json       # empty
    ├── pr-675-reviews.json               # empty
    └── online-research.md               # summarized + cited (R429)
```

Implemented and wired into the rest of the repository by:

- `src/world_model.rs` — the world-model feature: `Context`, `WorldModel`,
  `Action`, dependent `Statement`/`Dependency`, and
  `difference`/`predict`/`recalculate`/`merge_from`/`split_off`.
- `src/lib.rs` — `pub mod world_model;` plus re-exports of the public types.
- `tests/unit/issue_649_world_model.rs` — executable coverage of the links
  network, current→target diff, dependent-statement cascade, non-mutating
  prediction, and merge/split.
- `REQUIREMENTS.md` — rows **R428–R434** (R430).
- `README.md` and `ARCHITECTURE.md` — a discoverable reference to this case study.
- `tests/unit/docs_requirements_issue_649.rs` — pins the reference, the headings,
  and the requirement IDs so the case study cannot silently regress (R434).
- `changelog.d/` — a `minor` fragment recording the feature.
