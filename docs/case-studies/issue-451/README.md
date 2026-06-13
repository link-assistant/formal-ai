# Issue 451 Case Study

> **Status:** Reference added and best practices mapped in PR #452.
> **Type:** Documentation + enhancement (no behavioral code change required).
> **Primary source:** [Symbolic artificial intelligence — Wikipedia](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/451>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/452>
- **Best-practices mapping:** [`symbolic-ai-best-practices.md`](symbolic-ai-best-practices.md)
- **Online research (cited):** [`raw-data/online-research.md`](raw-data/online-research.md)

All raw artifacts referenced below live in [`raw-data/`](raw-data/).

---

## 1. Summary

Issue #451 asks the project to **reference the Wikipedia "Symbolic artificial
intelligence" article in the docs** and to **adopt the field's best known
practices — but expressed through the repository's own associative technological
stack** (Links Notation, the doublets link store, Wikidata/Wikipedia/Wiktionary/
WordNet grounding). It further asks for a **case study** with collected data,
**deep analysis**, **online research**, an **exhaustive requirement list**, and
**per-requirement solution plans with a survey of existing components**.

The central finding is that **`formal-ai` already _is_ a symbolic-AI system by
the article's own definition** — *"methods … based on high-level symbolic
(human-readable) representations of problems, logic, and search"* — so the issue
is satisfied not by adding a new subsystem but by (a) making the lineage
**explicit** in the top-level docs, and (b) **auditing** each best practice the
article names against the code that already realizes it, recording the few
genuine gaps as scoped future work rather than marketing claims.

The deliverables are therefore documentation-centric and fully traceable:

1. The Wikipedia article (plus *Neuro-symbolic AI*, *Semantic network*, and
   *Physical symbol system*) is cited from `README.md`, `VISION.md`, and
   `ARCHITECTURE.md`.
2. [`symbolic-ai-best-practices.md`](symbolic-ai-best-practices.md) maps **every
   technique family the article enumerates** to its associative-stack
   realization, with `path:symbol` evidence and an honest *applied / partial /
   proposed* status per row.
3. This case study collects the data, lists requirements **R298–R304**, and
   gives a solution plan and a prior-art survey for each.
4. `tests/unit/docs_requirements.rs::issue_451_symbolic_ai_reference_documents_are_present_and_traceable`
   pins all of the above so the reference and the mapping cannot silently
   regress.

---

## 2. Collected Data

The raw, third-party captures (exempt from authored-prose lints) are archived
under [`raw-data/`](raw-data/):

| File | What it is |
|---|---|
| [`raw-data/issue-451.json`](raw-data/issue-451.json) | The issue as filed (`gh issue view 451 --json …`). Labels: `documentation`, `enhancement`. |
| [`raw-data/issue-451-comments.json`](raw-data/issue-451-comments.json) | Issue comment thread — empty (`[]`); the issue body is the sole specification. |
| [`raw-data/pr-452.json`](raw-data/pr-452.json) | The draft pull request this work lands in. |
| [`raw-data/online-research.md`](raw-data/online-research.md) | Summarized-and-cited research: the article's definition, the boom/winter history, the technique taxonomy, named strengths/weaknesses/controversies, neuro-symbolic framing, the semantic-network ↔ associative-store mapping, and 2024–2026 survey literature. |

Per [NON-GOALS.md](../../../NON-GOALS.md) (*"Research notes should not copy large
external texts; they should summarize and cite sources"*), the research file
quotes only short definitional phrases and links every claim to its source.

---

## 3. Holistic Requirements

Every requirement extracted from the issue body (the issue has no comments, so
the body is the complete specification). These are recorded in
[`REQUIREMENTS.md`](../../../REQUIREMENTS.md) as **R298–R304** under
*"Issue #451 Symbolic AI Reference And Best Practices"*.

| ID | Requirement (verbatim intent) | Status |
|---|---|---|
| **R298** | Reference `https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence` in the project documentation. | Done — cited from `README.md`, `VISION.md`, `ARCHITECTURE.md` §17. |
| **R299** | Use **all best known practices** listed in that article *and others in the domain*, applied through the repository's **associative technological stack**. | Done — [`symbolic-ai-best-practices.md`](symbolic-ai-best-practices.md) maps every technique family to existing code; 3 genuine gaps are scoped as proposed work (§6). |
| **R300** | **Collect issue-related data** into `docs/case-studies/issue-451/`. | Done — see §2 and [`raw-data/`](raw-data/). |
| **R301** | Do a **deep case-study analysis**, including **searching online** for additional facts and data. | Done — §4 (analysis) + [`raw-data/online-research.md`](raw-data/online-research.md) (8 cited sources incl. 2024–2026 surveys). |
| **R302** | **List each and all requirements** from the issue. | Done — this table (R298–R304) plus the prose enumeration in this section. |
| **R303** | Propose **possible solutions and solution plans for each requirement**, checking **known existing components/libraries** that solve a similar problem or can help. | Done — §6 (per-requirement plans) + §7 (prior-art survey). |
| **R304** | **Plan and execute everything in the single PR #452.** | Done — every artifact above plus the regression test, changelog fragment, and doc edits land in PR #452. |

### Why these seven and not more

The issue body is one paragraph of intent. R298 is the literal title ask; R299
is the "best practices" sentence; R300–R301 are the "collect data … deep case
study … search online" sentence; R302 is "list of each and all requirements";
R303 is "propose possible solutions … check existing components"; R304 is "plan
and execute everything in this single pull request". No requirement is implied
beyond these without over-reading the text.

---

## 4. Deep Analysis — `formal-ai` as a Symbolic AI system

### 4.1 The project sits inside the article's definition, not adjacent to it

The article defines Symbolic AI (a.k.a. classical AI, logic-based AI, or
retrospectively **GOFAI**) as *"the collection of all methods in artificial
intelligence research that are based on high-level symbolic (human-readable)
representations of problems, logic, and search."* `README.md`'s first line
describes `formal-ai` as *"a symbolic, deterministic assistant … without
neural-network inference."* These are the same claim. The project is a member of
the category the issue points at, which is why "apply the best practices" is an
**audit-and-name** task, not a "bolt on a reasoner" task.

### 4.2 The physical symbol system hypothesis is the project's operating premise

Newell & Simon's hypothesis (1976) — *a physical symbol system has the necessary
and sufficient means for general intelligent action* — is exactly what
`VISION.md` operationalizes when it says the *"associative network of links"* **is**
the intelligence: symbols (links) are created, combined, and rewritten by formal
rules, and nothing else is consulted. There is no hidden weight vector; the
system of record is an inspectable, append-only graph.

### 4.3 The associative store **is** a semantic network

A *semantic network* is *"a knowledge base that represents semantic relations
between concepts in a network"* — vertices are concepts, labeled edges are
relations — and the article lists RDF triples and conceptual graphs as semantic
networks with *"expressive power equal to or exceeding standard first-order
predicate logic."* `formal-ai`'s doublet store is precisely this structure, and
**Links Notation is its human-readable serialization**. The classical-AI name
for the project's central data structure is "semantic network"; the project's
name for it is "associative network of links." Establishing that synonymy is the
single most useful outcome of this issue, because it lets every downstream best
practice be located by its classical name.

### 4.4 Where it sits on the field's two axes

- **Neats vs. scruffies** (the article's first controversy): `formal-ai` is
  deliberately **neat-leaning** — a single universal solver loop and a small set
  of typed rule shapes — but **scruffy-pragmatic** about knowledge, treating
  Wikidata/Wikipedia/Wiktionary/WordNet as a cached, ever-growing knowledge base
  rather than hand-encoding a Cyc-style ontology.
- **Symbolic vs. connectionist** (the second controversy): the project is at the
  **pure-symbolic** end (`NON-GOALS.md`: *"GPU-required neural inference is not a
  project target"*). The current field consensus the article reports —
  *integration, not winner-take-all* — is acknowledged by the project's one
  deliberate borrowing: a softmax-**temperature** selector re-applied to
  **discrete, Wikidata-anchored interpretations** (`VISION.md`, "Formalization
  And Temperature"). That is a symbolic re-implementation of a neural mechanism,
  pointed the opposite way from the usual neuro-symbolic hybrids.

### 4.5 What the 2024–2026 literature adds

The recent surveys (IEEE TPAMI 2025; arXiv 2411.04383; arXiv 2504.07640;
ScienceDirect 2025 — see the research file) converge on **LLM + knowledge graph**
as the most promising route to *"trustworthy, explainable, and interoperable"*
AI, and note that **explainability scales with the readability of the
neural↔symbolic intermediate representation**. `formal-ai`'s intermediate
representation is Links Notation **text** — maximally readable by construction —
and its knowledge graph is the associative store. The project's bet is that the
*graph + deterministic reasoning* half can stand alone for the task classes it
targets, while keeping the door open to the neural half later. Nothing in the
2024–2026 literature contradicts the architecture; it corroborates the emphasis
on a readable symbolic substrate.

---

## 5. Best Practices → Associative Stack (overview)

The full mapping with evidence and status is in
[`symbolic-ai-best-practices.md`](symbolic-ai-best-practices.md). Summary:

| Article technique family | Associative-stack realization | Status |
|---|---|---|
| Symbolic (human-readable) representation | Links Notation `.lino`; doublets | applied |
| Knowledge representation — semantic networks | the doublet link store itself | applied |
| Knowledge representation — ontologies | Wikidata/Wikipedia/Wiktionary/WordNet grounding + `sources-registry.lino` | applied |
| Production rules (if–then) | substitution rules (`when … then …`), intent-routing rule book | applied |
| Inference — forward/backward chaining | universal solver loop + `proof_engine` | applied |
| Search & planning (means-ends, STRIPS-like) | the 11-step decompose→test→synthesize→combine loop | applied |
| Automated theorem proving | `src/proof_engine/` decision procedures | applied |
| Reasoning under uncertainty (Bayes/Markov) | `src/probability.rs` (symbolic evidence) | applied |
| Explainability / provenance | append-only event log + `trace:` pointers | applied |
| Knowledge ⟂ procedure separation | `data/seed/*.lino` vs. `src/` ("Data Is The Interface") | applied |
| Symbolic ML — rule synthesis / ILP | `src/rule_synthesis.rs`, `skill_compiler.rs` | applied |
| Case-based reasoning | history lookup / answer reuse | applied |
| Truth maintenance / non-monotonic | append-only history + retraction discipline | applied |
| Meta-level reasoning (Soar-style) | budget-aware strategy selection in the solver | partial |
| Constraint solving / SAT | calculator delegation; no general constraint engine yet | **gap → proposed** |
| Common-sense / frame problem | bounded decomposition + public-KB grounding | partial (mitigated) |
| Neuro-symbolic integration | temperature selector over discrete interpretations | partial (by design) |

"Applied" rows are backed by `path:symbol` citations in the mapping document.
The three non-"applied" rows are the honest residue and drive the solution plans
below.

---

## 6. Solution Plans (per requirement)

Each plan names the chosen approach and the existing component it reuses, per
R303. The survey those choices draw on is §7.

### R298 — Reference the article in docs
**Approach (chosen, done):** cite the article where a reader forms their mental
model — `README.md`'s opening framing, a `VISION.md` paragraph naming the
"associative network of links" as a *semantic network*, and `ARCHITECTURE.md`
§17 *References* (alongside the existing doublets/Wikidata/Wikipedia entries).
Add the three closely-related articles (*Neuro-symbolic AI*, *Semantic network*,
*Physical symbol system*) so the lineage is navigable.
**Alternative considered:** a single mention in one file — rejected because the
three documents serve different readers (newcomer, contributor, integrator) and
the article is relevant to all three.

### R299 — Apply all best practices via the associative stack
**Approach (chosen, done):** an explicit audit table
([`symbolic-ai-best-practices.md`](symbolic-ai-best-practices.md)) with one row
per technique family the article names, each tied to the code that realizes it
and labeled *applied / partial / proposed*. This both **demonstrates** the
practices already in place and **scopes** the gaps without overclaiming.
**Existing components reused:** the project's own modules (`solver.rs`,
`proof_engine/`, `probability.rs`, `substitution.rs`, `rule_synthesis.rs`,
`knowledge.rs`) — the best practices are already implemented; the work is to
name and cite them.
**Proposed (gap) work, scoped not deferred-silently:**
- *Constraint solving / SAT* — the cleanest fit is the existing **delegated
  decision-procedure pattern** (the way `link-calculator` is called): a future
  associative package could wrap a Rust SAT/CSP crate (e.g.
  [`splr`](https://crates.io/crates/splr), [`varisat`](https://crates.io/crates/varisat))
  behind the same "formalize → delegate → trace" boundary `proof_engine` already
  uses. Noted as future work in the mapping doc, not claimed as present.
- *Meta-level reasoning* — partially present (budget-aware candidate selection);
  a fuller Soar-style agenda would extend `solver_diagnostics`.
- *Common-sense / frame problem* — inherent to symbolic AI; mitigated (not
  solved) by bounded decomposition depth and on-demand public-KB grounding.

### R300 — Collect issue data
**Approach (chosen, done):** archive issue/PR JSON + the (empty) comment thread
under `raw-data/`, matching the layout every prior case study uses (e.g.
issue-408, issue-195). **Existing component reused:** the `gh` CLI JSON export
already used across the repo's case studies.

### R301 — Deep analysis + online research
**Approach (chosen, done):** §4 here synthesizes the analysis;
[`raw-data/online-research.md`](raw-data/online-research.md) holds the cited
research, including 2024–2026 survey literature beyond the single article.
**Existing component reused:** the case-study `raw-data/online-research.md`
convention introduced by issue-195/issue-408.

### R302 — List all requirements
**Approach (chosen, done):** §3's R298–R304 table plus the "why these seven"
justification, mirrored into `REQUIREMENTS.md`.

### R303 — Solution plans + component survey
**Approach (chosen, done):** this section plus §7.

### R304 — Single PR
**Approach (chosen, done):** all artifacts land in PR #452: the three doc edits,
this case study, the mapping doc, the `raw-data/` captures, the
`REQUIREMENTS.md` rows, the regression test, and the changelog fragment.

---

## 7. Existing Components / Prior Art Surveyed (R303)

What the field already built for each best practice, and what `formal-ai` reuses
versus re-expresses in its associative stack.

### Knowledge representation
- **WordNet / Open English WordNet** — lexical semantic network. *Already
  ingested* (`REQUIREMENTS.md` R285; `data/cache/wordnet/`). Reused as a grounded
  ontology source, not re-implemented.
- **ConceptNet, YAGO, DBpedia, Wikidata** — large public semantic networks /
  knowledge graphs. Wikidata is *already* the P/Q-ID anchor layer; ConceptNet is
  a candidate future source registered the same way (`sources-registry.lino`).
- **RDF / RDFS / OWL, conceptual graphs** — the standardized semantic-network
  serializations. Links Notation is the project's serialization; the mapping doc
  notes the equivalence so an RDF export remains a natural future bridge.
- **Cyc** — the canonical hand-built common-sense ontology. Surveyed as the
  cautionary "scruffy" extreme: `formal-ai` deliberately avoids a decades-long
  hand-encoding effort by caching public knowledge on demand.

### Inference engines / rule systems
- **CLIPS, Jess, Drools, OPS5** — production-rule (forward-chaining) engines.
  `formal-ai`'s `when … then …` substitution rules + intent-routing book are the
  same idea expressed over doublets; no external engine is embedded because the
  rule surface is intentionally small and data-driven.
- **Prolog (SWI, GNU)** — Horn-clause backward chaining with unification. The
  `proof_engine` covers the decision-procedure niche; Prolog-style unification is
  noted as a future backend (`relative-meta-logic`).

### Theorem proving / SAT / constraints
- **Vampire, E, Z3, Prover9, ACL2** — ATP/SMT systems. `src/proof_engine/`
  occupies this role with built-in decision procedures (propositional truth
  tables, quantifier-free affine real arithmetic — `REQUIREMENTS.md` R18).
- **MiniSat / Glucose / `splr` / `varisat`** — SAT/CSP solvers. *Not yet
  integrated*; the proposed delegated-package plan (R299 gap) names these as the
  reuse target rather than a re-implementation.

### Uncertainty
- **Bayesian networks (Pearl), HMMs, Markov logic networks, ProbLog** —
  probabilistic reasoning. `src/probability.rs` provides symbolic
  `BayesianEvidence` and `MarkovTransition` records (`REQUIREMENTS.md` R6) — the
  associative, append-only re-expression of these ideas.

### Symbolic ML
- **ID3/C4.5, version spaces, inductive logic programming, case-based
  reasoning** — interpretable learning. `src/rule_synthesis.rs` +
  `skill_compiler.rs` synthesize rules from data/examples; history reuse is the
  case-based-reasoning analog.

### Neuro-symbolic frameworks
- **DeepProbLog, Logic Tensor Networks, Scallop, Neural Theorem Prover** — the
  hybrid frontier. Surveyed (research file §7) and intentionally *out of current
  scope* (no neural half), but the temperature-selector borrowing shows the
  project tracks the integration literature.

### The associative stack's own components (reused, not re-bought)
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs) —
  native semantic-network store.
- [`linksplatform/doublets-web`](https://github.com/linksplatform/doublets-web) —
  browser mirror.
- [`link-assistant/calculator`](https://github.com/link-assistant/calculator) —
  the *existing template* for delegating to an external decision procedure, which
  the proposed SAT/CSP integration would copy.

**Net conclusion:** for every best practice the article names, either a project
component already realizes it (and is now cited), or a specific, named external
component is the reuse target for the gap — no best practice is left both
unimplemented and unplanned.

---

## 8. Risks

The article's named **weaknesses of symbolic AI** are real risks for any system
in this category. Each is matched to the project's existing mitigation; none is
claimed "solved" (per `NON-GOALS.md`).

| Article weakness | Risk for `formal-ai` | Mitigation in the stack |
|---|---|---|
| **Knowledge-acquisition bottleneck** | Hand-encoding a large knowledge base is slow and goes stale. | Small seed + **on-demand growth** from public KBs treated as a bounded cache (`knowledge.rs`, R289–R290 cache caps). |
| **Common-sense reasoning** | Implicit world knowledge is hard to formalize. | Ground in public KBs rather than hand-author; accept bounded coverage and degrade to an explicit "unknown" rather than guess. |
| **Frame / qualification problem** | Enumerating all action preconditions/effects is intractable. | Bounded decomposition depth; the solver records what it did *not* establish rather than assuming. |
| **Brittleness** | Sharp off-domain failure. | Deterministic "unknown" fallback + traceable reason, instead of a confident wrong answer. |
| **Combinatorial explosion** | Search blows up. | Budget-aware candidate selection (smallest sufficient draft) and a small typed rule surface. |

Documenting these honestly is itself a best practice the article implies and the
project's NON-GOALS demand: *"Case studies should not become marketing pages."*

---

## 9. Files

```
docs/case-studies/issue-451/
├── README.md                         # this analysis
├── symbolic-ai-best-practices.md     # full best-practice → associative-stack mapping (R299)
└── raw-data/                         # third-party captures (lint-exempt)
    ├── issue-451.json                # the issue as filed
    ├── issue-451-comments.json       # comment thread (empty)
    ├── pr-452.json                   # the pull request
    └── online-research.md            # summarized + cited online research (R301)
```

Wired into the rest of the repository by:

- `README.md`, `VISION.md`, `ARCHITECTURE.md` — the Wikipedia reference (R298).
- `REQUIREMENTS.md` — rows **R298–R304** (R302).
- `tests/unit/docs_requirements.rs::issue_451_symbolic_ai_reference_documents_are_present_and_traceable`
  — pins the reference, the mapping headings, and the requirement IDs.
- `changelog.d/` — a `minor` fragment recording the reference + mapping.
