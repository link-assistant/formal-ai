## Issue #649 World Models And Contexts

Issue [#649](https://github.com/link-assistant/formal-ai/issues/649) asks Formal
AI to reason with **symbolic world models**: throughout a dialogue build a
**current-state** and a **target-state** world model, expose their difference,
let the user and agent **synchronize** the target, **merge** and **split**
context models where **each context is a links network** (explicitly *not*
embeddings), use [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic)
so **statements are dependent** and **any change recalculates all statement
probabilities**, and ultimately **predict the consequences of an action**. Its
final paragraph mandates the concrete deliverable: collect the issue data into a
case-study directory, do a deep analysis with online research, list every
requirement, propose a solution plan per requirement while surveying existing
components/libraries, and land it all in the single PR
[#675](https://github.com/link-assistant/formal-ai/pull/675). The case study
under `docs/case-studies/issue-649` finds the associative stack already supplies
every substrate (links network, fixpoint rewrite, the RML kernel, symbolic
probability, context merge) so the feature is an audit-and-wire design task
rather than green-field infrastructure; the per-issue requirements R649-01 …
R649-19 are enumerated in `docs/case-studies/issue-649/requirements.md`.

| ID | Requirement | Status |
| --- | --- | --- |
| R428 | Preserve the issue #649 source material, prepared PR #675 state, empty comment/review threads, and online research under a dedicated case-study directory. | Implemented by `docs/case-studies/issue-649/raw-data/` and protected by `tests/unit/docs_requirements_issue_649.rs`. |
| R429 | Produce a deep case-study analysis of the symbolic world-model request, including online research beyond the source video. | Implemented by `docs/case-studies/issue-649/README.md` and `docs/case-studies/issue-649/raw-data/online-research.md` (STRIPS/PDDL, situation calculus, JTMS/ATMS, AGM belief revision, and the world-model literature, all cited). |
| R430 | Enumerate each and every requirement of the issue. | Implemented by `docs/case-studies/issue-649/requirements.md` (conceptual R649-01 … R649-14 plus meta-deliverable R649-15 … R649-19). |
| R431 | Map every world-model / context concept to its associative-stack realization with honest Realized/Partial/Proposed status and `path:symbol` evidence. | Implemented by `docs/case-studies/issue-649/world-model-mapping.md` (3 realized substrate rows, 8 partial, 3 proposed across 14 concepts). |
| R432 | Propose a solution plan for each requirement and survey known existing components/libraries that solve a similar problem. | Implemented by `docs/case-studies/issue-649/solution-plans.md` (per-requirement plans reusing `SubstitutionGraph`, `relative_meta_logic`, `probability`, `memory_sync`, `meta_frame`, plus a STRIPS/ATMS/JTMS/AGM/JEPA prior-art survey). |
| R433 | Plan and execute every deliverable in the single PR #675. | Implemented by the full `docs/case-studies/issue-649/` tree plus this matrix section, the changelog fragment, and the traceability test. |
| R434 | Protect the issue #649 case study with a documentation-traceability regression test. | Implemented by `tests/unit/docs_requirements_issue_649.rs`, registered in `tests/unit/mod.rs`. |
