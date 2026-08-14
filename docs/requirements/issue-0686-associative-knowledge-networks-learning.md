## Issue #686 Associative Knowledge Networks Learning

Issue [#686](https://github.com/link-assistant/formal-ai/issues/686) asks Formal
AI to keep a **persistent** version of **meta-language expressions** saved in
**associative links networks** — applying the best practices of the cited paper
[arXiv 2512.00590](https://huggingface.co/papers/2512.00590) (*Wikontic*): not only
operate on facts but **persist** them, **count usages (reads) and changes
(writes)**, ensure the **most frequently used or changed data persists longer**,
**calculate usages from incoming and outgoing links**, and **keep everything as a
link / link network — not graph, not edges, not vertices**. Its final paragraph
mandates the concrete deliverable: collect the issue data into a case-study
directory, do a deep analysis with online research, list every requirement, propose
a solution plan per requirement while surveying existing components/libraries, and
land it all in the single PR
[#689](https://github.com/link-assistant/formal-ai/pull/689). The case study under
`docs/case-studies/issue-686` finds the associative stack already supplies most of
the substrate (a links network in `SubstitutionGraph`, one-node-per-meaning via
`stable_id`, and an LFU read-count eviction policy in `dreaming::usage_counts`), so
the implementation must connect that substrate to durable memory/dreaming and
carry the paper's qualifier, validation, normalization, incremental-ingestion,
and multi-hop practices into the symbolic runtime; the per-issue requirements are enumerated
in `docs/case-studies/issue-686/requirements.md`.

| ID | Requirement | Status |
| --- | --- | --- |
| R445 | Preserve the issue #686 source material, prepared PR #689 state, all conversation/review APIs, and online research under a dedicated case-study directory. | Implemented by `docs/case-studies/issue-686/raw-data/` and protected by `tests/unit/docs_requirements_issue_686.rs`. |
| R446 | Produce a deep case-study analysis of the persistence request, including online research beyond the cited paper. | Implemented by `docs/case-studies/issue-686/README.md` and `docs/case-studies/issue-686/raw-data/online-research.md` (Wikontic, AriGraph, LFU/LRU cache replacement, reference counting, degree centrality, all cited). |
| R447 | Enumerate each and every requirement of the issue. | Implemented by `docs/case-studies/issue-686/requirements.md` (R686-01 … R686-18, including the paper pipeline, runtime/Agent CLI integration, evidence, and delivery). |
| R448 | Map every persistence concept to its associative-stack realization with honest Realized/Partial/Done status and `path:symbol` evidence. | Implemented by `docs/case-studies/issue-686/persistence-mapping.md` (13 done and 2 realized substrate across 15 concepts). |
| R449 | Propose a solution plan for each requirement and survey known existing components/libraries that solve a similar problem. | Implemented by `docs/case-studies/issue-686/solution-plans.md` (per-requirement plans reusing `SubstitutionGraph`, `stable_id`, `dreaming`, `storage_policy`, `world_model`, plus a Wikontic/AriGraph/LFU/reference-counting/degree-centrality prior-art survey). |
| R450 | Implement a usage-weighted associative persistence store: persist meta-language expressions as content-addressed nodes in a links network, count reads and writes, derive usage from incoming and outgoing link degree, and evict the least-used first — keeping everything as a link. | Implemented by `src/associative_persistence.rs` (`AssociativeMemory`, `PersistedExpression`, `RetentionWeights`, `ScoredExpression`) and covered by `tests/unit/issue_686_associative_persistence.rs`. |
| R451 | Plan and execute every deliverable in the single PR #689. | Implemented by the full `docs/case-studies/issue-686/` tree plus this matrix section, the changelog fragment, and the traceability test. |
| R452 | Protect the issue #686 case study with a documentation-traceability regression test. | Implemented by `tests/unit/docs_requirements_issue_686.rs`, registered in `tests/unit/mod.rs`. |
| R453 | Apply all transferable Wikontic pipeline practices: qualifier-preserving candidates, alignment validation with retained warnings, alias normalization/deduplication, incremental extension, and bounded multi-hop retrieval. | Implemented by `AssociativeMemory::from_memory_events`, expression qualifiers/validation issues, stable-id normalization, and `recall_related`; covered by issue-686 unit tests. |
| R454 | Make read/write/link-weighted retention affect the durable runtime and automatic dreaming policy, rather than only a standalone demonstration store. | Implemented by durable `MemoryEvent::write_count` serialization/sync/substitution and `dreaming::usage_counts` delegating to `AssociativeMemory::retention_score`. |
| R455 | Keep the browser memory mirror interoperable with the native memory format and write accounting. | Implemented by `src/web/memory.js` support for `accessCount`, `writeCount`, and substitution write increments. |
| R456 | Execute an associative auto-learning task through Formal AI via Agent CLI and derive its artifact from persisted memory. | Implemented by `agentic_coding::associative_learning`, the generalized document recipe, `data/meta/associative-learning-case.lino`, unit/driver coverage, and external Agent CLI evidence. |
| R457 | Reproduce and prevent the stable-id mutation defect where a write incremented its counter but retained stale text. | Implemented by updating text in `persist_identified` and `writing_a_changed_identified_expression_persists_the_new_value`. |
| R458 | Incorporate the maintainer's PR feedback, current mainline architecture, and all issue/PR comment types into the case study and implementation. | Implemented by the refreshed raw captures, merge commit, revised research/mapping/plans, and expanded requirements R686-01 … R686-18. |
