# Issue 649 — Solution Plans & Prior-Art Survey

Per **R649-18**, this file gives a concrete solution plan for each requirement in
[`requirements.md`](requirements.md) and, for each, names the **existing
component or external library** it reuses. The guiding finding of the case study
(see [`README.md`](README.md)) is that the world-model feature is an
**audit-and-wire** task over the associative stack, so every plan below reuses
`SubstitutionGraph` (fixpoint recompute), `LinkStore` (links network),
`relative_meta_logic` (RML kernel), and `probability` (Markov/Bayes evidence)
rather than new infrastructure.

The classical prior art each plan mirrors is documented and cited in
[`raw-data/online-research.md`](raw-data/online-research.md); the short tags below
(STRIPS, ATMS, JTMS, AGM, RML) point into that file.

---

## Architecture in one paragraph (implemented in `src/world_model.rs`)

PR #675 landed a first-class **`Context`** = `{ id, links: SubstitutionGraph,
statements: BTreeMap<String, Statement> }`, where a `Statement` carries an RML
`TruthValue` and its dependency edges are `Dependency{on, stance}` records
mirrored into the same graph. A **`WorldModel`** holds three contexts —
`current`, `target`, and the merged **general** context. An **`Action`** is a set
of link edits (add/delete, i.e. STRIPS effects); *predicting its consequence* is
`Context::predict` — apply the action to a **clone** of a context, recalculate,
and diff. *Recalculation* is `Context::recalculate`, a bounded fixpoint where a
changed `TruthValue` re-fires `StatementAssessment` for every dependent statement
(JTMS-style). Merge/split are `Context::merge_from` and `Context::split_off`,
scoped to a context's subgraph (ATMS-style multiple contexts). Nothing here needs
a new store, a neural model, or an external solver.

---

## Per-requirement plans

### R649-01 — Links networks as meaning representation
**Plan (Realized):** none required — the repository is already links-based; the
world model is built *on* `LinkStore`/Links-Notation, so it inherits inspectability
for free. **Reuses:** `src/link_store.rs::LinkStore`, `src/links_format.rs`.
**Prior art:** semantic networks / RDF triples (issue-451); the issue's explicit
rejection of embeddings (RML, online-research §1).

### R649-02 — Current-state world model
**Plan (Proposed):** wrap the already-projected current state in a
`Context{current}`: seed its `SubstitutionGraph` from `SyncStore`'s append-only
log via the existing `memory_event_to_link_record` projection, and keep it live
as the dialogue advances. **Alternative considered:** a brand-new state store —
rejected because `VISION.md`/`R61` already mandate "derive current state from
logged events," so a second source of truth would drift. **Reuses:**
`MemoryStore`, `SyncStore::record_chat_exchange`, `SubstitutionGraph`.
**Prior art:** STRIPS *initial state* (a set of true atoms).

### R649-03 — Target-state world model
**Plan (Proposed):** build `Context{target}` from `IntentFormalization` +
`ChangeRequest::derived_requirement` at formalization time, in the *same doublet
shape* as the current context so the two are directly diffable. **Reuses:**
`intent_formalization.rs`, `change_request.rs`, `program_plan.rs::resolved_task`.
**Prior art:** STRIPS *goal* (a condition over atoms).

### R649-04 — Difference between current and target
**Plan (Done):** implemented as `Context::difference(target) -> ContextDiff {
to_add, to_remove, conflicting }` over the two contexts' state-link sets, with
`ContextDiff::links_notation` for glass-box rendering; `WorldModel::difference`
exposes it at any dialogue stage and `target_reached` reports an empty diff.
**Reuses:** `SubstitutionLink` ordering; substrate `SubstitutionTraceReport`,
`meta_frame::NeedLedger`. **Prior art:** STRIPS *goal − state* delta (the add/
delete lists an action must achieve).

### R649-05 — User states the target
**Plan (Proposed):** route "I want …" / change instructions through
`derive_requirement` into `Context{target}` edits, so a user utterance becomes an
explicit target-model mutation. **Reuses:** `change_request.rs`,
`intent_formalization.rs`. **Prior art:** goal specification in PDDL problems.

### R649-06 — Explain current & target exactly
**Plan (Proposed):** extend `self_explanation.rs` to render each context as
Links-Notation plus a plain-language gloss, reading strictly from the append-only
trace so the explanation is exact and reproducible (glass box). **Reuses:**
`self_explanation.rs`, `event_log.rs` `trace:` pointers, `links_format.rs`.
**Prior art:** explainability/provenance best practice (issue-451 §Explainability).

### R649-07 — Synchronize the target (agent ⇄ user)
**Plan (Proposed):** a confirm/amend loop — after building `Context{target}`, use
`question_generation.rs` to surface it back to the user ("here is what I think you
want; correct me"), applying accepted amendments as target edits. **Reuses:**
`question_generation.rs`, R649-05 plumbing. **Prior art:** AGM *revision*
(rational update of a belief set under new input).

### R649-08 — Merge a context into the general world model
**Plan (Done):** implemented as `Context::merge_from(other)` — union the other
context's state links and statements (by id, last-writer-wins) then recalculate;
`WorldModel::commit_current_to_general` folds the dialogue context into the shared
general context. **Reuses:** the `merge_union_by_id` union-by-id semantics.
**Prior art:** ATMS combining assumption sets across contexts.

### R649-09 — Split contexts
**Plan (Done):** implemented as `Context::split_off(child_id, statement_ids)` —
carve a child context holding copies of the named statements plus every
state link that references one of them, leaving the parent intact so the two can
diverge independently. **Reuses:** substrate `meta_frame.rs` splitters + `WorkUnit`
as the linguistic-split analog. **Prior art:** ATMS maintaining multiple
simultaneous contexts.

### R649-10 — Each context is a links network
**Plan (Done):** implemented as `Context{ id, links: SubstitutionGraph,
statements }`; the network is the existing graph and the statement layer mirrors
into it (`Context::links_notation` serializes the whole context). **Reuses:**
`SubstitutionGraph`. Distinct from the *linguistic*
`concepts.rs::resolve_context_label`. **Prior art:** semantic network as the
universal container.

### R649-11 — Dependent statements
**Plan (Done):** implemented as `Dependency{on, stance}` edges on each `Statement`
(mirrored into the context's `SubstitutionGraph` via `supports:`/`contradicts:`
links); a statement's `TruthValue` becomes graph-visible state. **Reuses:**
`SubstitutionGraph::{insert_link, remove_link}`,
`relative_meta_logic::StatementAssessment`. **Prior art:** JTMS *justifications*;
RML statements-relative-to-statements.

### R649-12 — Recalculate all probabilities on change
**Plan (Done):** implemented as `Context::recalculate` — each context edit
(`add_statement`, `apply_action`, `merge_from`) runs a bounded relaxation that
re-invokes `StatementAssessment::assess` for every statement, feeding it the
current truth of its dependencies as synthesized full-trust evidence, until the
snapped values reach a fixpoint (`RecalculationReport{iterations, converged,
updated}`). Values snap to the RML decimal grid for reproducibility, and the pass
bound guarantees termination on negative-feedback cycles. **Reuses:**
`relative_meta_logic` aggregators and `StatementAssessment`. **Prior art:** JTMS
re-evaluation of affected beliefs; RML query-time re-evaluation.
**Libraries surveyed:** upstream [`relative-meta-logic`](https://github.com/link-foundation/relative-meta-logic)
(JS+Rust) — *reused via the in-repo port* rather than added as a dependency, to
keep the engine WASM-safe and byte-reproducible (same rationale as the issue-451
SAT decision).

### R649-13 — Use relative-meta-logic
**Plan (Realized):** the kernel is already ported to
`src/relative_meta_logic.rs`; the world model consumes it directly. **Reuses:**
`relative_meta_logic::{TruthValue, Aggregator, StatementAssessment}`.
**Prior art:** the upstream RML repo.

### R649-14 — Predict consequences of an action
**Plan (Done):** implemented as `Context::predict(action)` — clone the context,
`apply_action` the action's add/delete link edits to the clone, recalculate via
R649-12, and return a `Prediction{added, removed, statement_changes, result}`
(state links added/removed plus every statement whose probability moves), never
mutating the real world model. `WorldModel::predict` runs it against the current
state. This composes R649-04 + R649-12. **Reuses:** the R649-04 diff, the R649-12
cascade. **Prior art:** STRIPS successor state via add/delete effects;
probabilistic planning (distribution over next states, online-research §3).

### R649-15 … R649-19 — Meta-deliverables
**Plan (Done):** the `gh`-exported `raw-data/` captures (R649-15); this analysis
plus the cited `online-research.md` (R649-16); the `requirements.md` table
(R649-17); this file and the prior-art survey below (R649-18); all landed in PR
#675 together with the `src/world_model.rs` implementation, its
`tests/unit/issue_649_world_model.rs` coverage, `REQUIREMENTS.md` rows R428–R434,
a changelog fragment, and the `tests/unit/docs_requirements_issue_649.rs`
traceability test (R649-19). **Reuses:** the case-study conventions of issue-451
and issue-540.

---

## Existing Components / Prior Art Surveyed (R649-18)

What the field and the repo already built, and what the world model reuses vs.
re-expresses.

### The new module (PR #675)
- **`world_model`** (`src/world_model.rs`) — the feature itself: `Context`,
  `WorldModel`, `Action`, `Statement`/`Dependency`, `Prediction`, `ContextDiff`,
  `RecalculationReport`, and `Context::{difference, predict, recalculate,
  merge_from, split_off}`. A thin wiring layer over the components below.

### In-repo components (reused, not re-bought)
- **`SubstitutionGraph`** (`src/substitution.rs`) — mutable links graph with
  CRUD and `apply_rules` fixpoint rewriting. *The `world_model::Context` state
  container; its CRUD backs `assert_link`/`retract_link` and the statement-layer
  mirror.*
- **`relative_meta_logic`** (`src/relative_meta_logic.rs`) — the RML kernel
  (`TruthValue`, aggregators, `StatementAssessment`). *The per-statement logic.*
- **`probability`** (`src/probability.rs`) — `BayesianEvidence` /
  `MarkovTransition` symbolic evidence, on-demand re-aggregation. *Uncertain
  action effects and next-state weighting.*
- **`memory_sync`** (`src/memory_sync.rs`) — `merge_union_by_id`/`merge_event`.
  *Context merge.*
- **`meta_frame`** (`src/meta_frame.rs`) — `NeedLedger`, clause/span splitters,
  `WorkUnit`. *Context split and the diff/gap surface.*
- **`link_store` / `links_format` / `links_query`** — the links-network substrate
  and Links-Notation serialization. *Each context is one of these.*

### External formalisms (re-expressed in the associative stack)
- **STRIPS / PDDL** — the canonical *state + goal + add/delete effects* model.
  Re-expressed: current/target contexts are state/goal; an action is a set of
  link edits. Not embedded as a planner; the solver's decompose→test loop plays
  the search role.
- **Situation calculus** — situations produced by actions. Re-expressed as
  successive context snapshots; heavy reasoning stays in bounded `proof_engine`
  procedures rather than open resolution (its known weakness).
- **Truth Maintenance Systems (JTMS/ATMS)** — recompute beliefs on change; ATMS
  keeps *multiple contexts*. Re-expressed: dependency links + `apply_rules`
  fixpoint (JTMS), context objects with merge/split (ATMS).
- **AGM belief revision** — expansion/revision/contraction postulates.
  Re-expressed as the merge (expansion), target-sync (revision), and split
  (contraction) operations over contexts.
- **`relative-meta-logic`** (upstream) — probabilistic many-valued logic on LiNo.
  Reused via the in-repo port; the external repo remains the reference spec and
  the upgrade path (e.g. Belnap operators) if richer valence is needed.

### External world-model systems (surveyed, intentionally not adopted)
- **JEPA / V-JEPA 2 / energy-based planning** (LeCun; the video the issue cites)
  — learned-embedding world models. *Out of scope by construction*: the issue
  requires links networks *instead of* embeddings. Surveyed to keep the shared
  goal (predict consequences, plan) while diverging on representation, matching
  the repo's pure-symbolic `NON-GOALS.md` stance.

**Net conclusion:** for every requirement, either an in-repo component already
realizes it (and is cited) or a specific, named component is the concrete reuse
target for a *Proposed* wiring step — no requirement is left both unrealized and
unplanned.
