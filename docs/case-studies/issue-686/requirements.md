# Issue 686 — Holistic Requirements

Every requirement extracted from the issue body. The issue has **no comments**
([`raw-data/issue-686-comments.json`](raw-data/issue-686-comments.json) is `[]`),
so the body is the complete specification. Requirements are split into the
**conceptual feature** the issue describes (R686-01 … R686-07) and the
**meta-deliverable** it asks this PR to produce (R686-08 … R686-13).

Status legend: **Realized** — already present in the codebase and reused as-is;
**Partial** — a reusable precursor exists but not in the shape the issue wants;
**Proposed** — scoped future work with a concrete plan in
[`solution-plans.md`](solution-plans.md); **Done** — delivered by this PR.

## Conceptual feature requirements

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R686-01** | Apply the **best practices from the cited paper** (arXiv 2512.00590, *Wikontic*): normalize/deduplicate so **one meaning is one node**, and treat **node degree as the currency of retrieval/importance**. | Realized (substrate) — content-addressed `stable_id` already gives one node per meaning (`src/engine.rs`); degree is now a first-class signal in `src/associative_persistence.rs`. |
| **R686-02** | Keep a **persistent** version of **meta-language expressions** saved in **associative links networks** — not only *operate* on facts but *retain* them. | Done — `src/associative_persistence.rs::AssociativeMemory` persists each expression as a content-addressed node in an embedded `SubstitutionGraph`. |
| **R686-03** | **Count usages (reads)** per expression. | Done — `AssociativeMemory::{note_read, reads}`; a fresh expression starts at 0 reads. |
| **R686-04** | **Count changes (writes)** per expression. | Done — `AssociativeMemory::{persist, note_write, writes}`; asserting or re-asserting an expression is a write. |
| **R686-05** | The data **most frequently used or changed persists for longer** (usage-weighted retention / eviction). | Done — `AssociativeMemory::{retention_score, retention_ranking, eviction_order, evict_least_used, retain_most_used}` — an LFU-style policy that forgets the lowest-scored first. |
| **R686-06** | **Calculate usages based on incoming and outgoing links** (degree as an alternative usage signal). | Done — `AssociativeMemory::{in_degree, out_degree, degree, link_usage}`; both degree halves also feed `retention_score` with independent weights. |
| **R686-07** | **Keep everything as a link, or link network — not graph, not edges, not vertices.** | Done — associations are `SubstitutionLink` doublets in a `SubstitutionGraph`; `AssociativeMemory::links_notation` serializes expressions, reads, writes, and associations all as links. |

## Meta-deliverable requirements (this PR)

| ID | Requirement | Status |
|---|---|---|
| **R686-08** | **Collect the issue-related data** into `docs/case-studies/issue-686/`. | Done — [`raw-data/`](raw-data/). |
| **R686-09** | Do a **deep case-study analysis**, including **online research** for additional facts. | Done — [`README.md`](README.md) + [`raw-data/online-research.md`](raw-data/online-research.md). |
| **R686-10** | **List each and all requirements** from the issue. | Done — this file (R686-01 … R686-13). |
| **R686-11** | Propose **possible solutions and solution plans for each requirement**, checking **known existing components/libraries**. | Done — [`solution-plans.md`](solution-plans.md). |
| **R686-12** | Map each concept to its associative-stack realization with **honest status**. | Done — [`persistence-mapping.md`](persistence-mapping.md). |
| **R686-13** | **Plan and execute everything in the single PR** ([#689](https://github.com/link-assistant/formal-ai/pull/689)). | Done — every artifact in this directory plus the `src/associative_persistence.rs` implementation, its `tests/unit/issue_686_associative_persistence.rs` coverage, `REQUIREMENTS.md` rows R445–R452, the changelog fragment, and the traceability test. |

## Why these thirteen and not more

The issue body is three short intent paragraphs plus one meta paragraph.
R686-01 is the "use all the best practices from [the paper]" sentence; R686-02 is
the "persistent version of meta language expressions saved in associative links
networks / we not only operate with facts we persist them" sentence; R686-03 and
R686-04 are the "count usages (reads) and changes (writes)" clause; R686-05 is the
"most frequently used or changes should persistent for longer" sentence; R686-06
is the "calculate usages based on incoming and outgoing links" sentence; R686-07
is the "keep everything as a link, or link network, not graph, not edges, not
vertices" sentence. R686-08 … R686-13 are the sentences of the final meta
paragraph (collect data → deep analysis with online research → list requirements →
propose solution plans surveying existing components → do it all in this one PR),
plus the honest concept→stack mapping the project's case-study pattern requires.
No requirement is implied beyond these without over-reading the text.

## Relationship to the global requirement matrix

These per-issue IDs are the fine-grained specification. The
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) global matrix records this PR's
concrete deliverables as rows **R445–R452** under *"Issue #686 Associative
Knowledge Networks Learning"*, each pointing back into this directory.
