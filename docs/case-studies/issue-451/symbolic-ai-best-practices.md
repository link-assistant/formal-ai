# Symbolic AI best practices, expressed in the associative stack

This is the **R299 audit** for issue #451: every best practice the
[Symbolic AI article](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
enumerates (plus the engineering practices it implies and adjacent
domain sources), mapped to the part of `formal-ai`'s **associative technological
stack** that realizes it — with `path:symbol` evidence so each claim is
checkable.

The point of the issue is *not* to add a new reasoner. The project already
satisfies the article's definition of a symbolic-AI system (see the case-study
[`README.md`](README.md) §4). The point is to **name** each classical practice,
**locate** it in the code, and **honestly scope** the few that are not yet
present.

### Status legend

- **applied** — a shipped, cited component realizes the practice.
- **partial** — realized for the project's current scope, with a known boundary.
- **proposed** — a genuine gap; a specific external component is named as the
  reuse target (never silently deferred — see §9).

### Stack vocabulary ↔ classical vocabulary

| Classical (article) term | Associative-stack term | Where |
|---|---|---|
| Symbol | Link (doublet) | `linksplatform/doublets-rs` |
| Semantic network | Associative network of links | `VISION.md` line 3 |
| Symbol serialization (RDF/CG) | Links Notation (`.lino`) | `data/seed/**.lino` |
| Knowledge base | Seed + cached public KBs | `data/seed/`, `data/cache/` |
| Inference engine | Universal solver | `src/solver.rs` |
| Production rule | `when … then …` / `replace x y` | `src/substitution.rs` |

---

## 1. Knowledge representation

### 1.1 Symbolic, human-readable representation — **applied**
The article's defining trait: *"high-level symbolic (human-readable)
representations."* The stack's entire surface is human-readable Links Notation;
domain knowledge lives as reviewable `.lino` under `data/seed/` and is loaded by
`src/seed.rs`. There is no opaque tensor anywhere in the path
(`NON-GOALS.md`: *"GPU-required neural inference is not a project target"*).

### 1.2 Semantic networks — **applied**
*"A knowledge base that represents semantic relations between concepts in a
network."* This **is** the doublet store: vertices are concepts, edges are
relations. `VISION.md` line 3 names the live state an *"inspectable associative
network of links."* The article lists RDF triples and conceptual graphs as
semantic networks with *"expressive power equal to or exceeding standard
first-order predicate logic"*; Links Notation is the project's equivalent
serialization, which is why an RDF export is a natural (not yet built) bridge.

### 1.3 Ontologies & grounding — **applied**
The article lists WordNet/YAGO/DOLCE/OWL as ontology practice. The stack grounds
meanings in public ontologies rather than hand-authoring one:
- Wikidata P/Q-IDs as anchors, Wikipedia/Wiktionary per-language entries
  (`ARCHITECTURE.md` §17; `data/seed/sources-registry.lino`, `REQUIREMENTS.md`
  R287).
- Open English WordNet 2024 mirrored in-repo and reachable by meanings
  (`REQUIREMENTS.md` R285; `data/cache/wordnet/`).
- Total reference closure — every token resolves to a defined meaning or a
  grounded source (`REQUIREMENTS.md` R284; `tests/unit/total_closure.rs`). That
  closure discipline is the associative-stack analog of an ontology's
  well-formedness constraint.

### 1.4 Production rules (if–then) — **applied**
The article's CLIPS/OPS5/Jess/Drools family. The stack expresses them as
data-driven substitution rules over doublets: `src/substitution.rs` parses a
`when <condition>` guard (`src/substitution.rs:642`) and applies `replace x y`
/ `then` rewrites (the module header documents the issue #301 `replace x y`
primitive). Intent routing is itself a rule book (`data/seed/intent-routing.lino`)
and the rule kinds are catalogued in `ARCHITECTURE.md` §9.

### 1.5 Frames & scripts — **partial**
Minsky frames / Schank scripts (slots with defaults; stereotyped event
sequences) are not a named construct, but their function — structured concepts
with typed relations and role expectations — is carried by concept records
(`data/seed/concepts.lino`, `data/seed/concept-contexts.lino`) and role seeds.
Boundary: there is no inheritance-with-default-override frame system; concept
structure is explicit links rather than slot defaults.

---

## 2. Reasoning / inference

### 2.1 Forward / backward chaining — **applied**
Forward chaining (data → conclusions) is the substitution-rule engine firing
`when … then …` rules as facts appear; backward chaining (goal → subgoals) is
the universal solver decomposing an impulse into sub-impulses it must establish
(`src/solver.rs` steps 5–8; `record_decomposition`, `record_candidates`). The
solver is deterministic for a given config and impulse (`src/solver.rs` header).

### 2.2 Automated theorem proving — **applied**
The article's Vampire/E/ACL2 niche. `src/proof_engine/` provides a universal
proof/disproof engine with **delegated decision procedures**
(`src/proof_engine/decision.rs`): propositional truth-table enumeration and
quantifier-free affine real-arithmetic solving (`REQUIREMENTS.md` R18). The
delegated-procedure boundary is the reuse seam for §9's proposed SAT work.

### 2.3 Truth maintenance / non-monotonic reasoning — **applied**
The article's TMS/non-monotonic practice maps to the **append-only event log**:
`src/event_log.rs` ("Append-only event log for the universal solver … the answer
is … a projection of the log"). Beliefs are added as events; retraction is a new
event, never a destructive edit — the add-only history `VISION.md` mandates. This
is also the explainability substrate (§6.1).

### 2.4 Meta-level reasoning — **partial**
Soar-style "reasoning about how to reason" appears as budget-aware strategy
selection: the solver chooses the smallest sufficient candidate and can guess
under a content-hash-seeded draw only when permitted (`ARCHITECTURE.md` §6;
`src/solver_diagnostics.rs`). Boundary: there is no general meta-rule agenda;
strategy selection is built into the loop rather than itself rule-driven.

---

## 3. Search & planning

### 3.1 Means-ends analysis / decomposition planning — **applied**
GPS/STRIPS-style decomposition is the spine of the universal loop: *decompose
the problem into tasks, derive a test for each, draft candidates, select the
smallest sufficient draft, recombine* (`README.md` "Universal Problem-Solving
Algorithm"; `src/solver.rs` steps 5–11). This is classical reasoning-as-search
applied to impulses.

### 3.2 SAT / constraint solving — **proposed** (gap)
The article devotes a section to DPLL/CDCL/WalkSAT and constraint solving. The
stack currently delegates only arithmetic (to `link-calculator`); there is no
general SAT/CSP engine. **Reuse target:** wrap a Rust solver
([`splr`](https://crates.io/crates/splr) or
[`varisat`](https://crates.io/crates/varisat)) behind the same
"formalize → delegate → trace" boundary `src/proof_engine/decision.rs` already
uses for arithmetic. Scoped in §9; not claimed present.

---

## 4. Reasoning under uncertainty

### 4.1 Bayesian / Markov, symbolically — **applied**
The article's Pearl/HMM/Markov-logic family. `src/probability.rs` stores
append-only probabilistic **evidence** as Links Notation records and ranks
candidates with `ProbabilityModel::BayesianEvidence` and
`ProbabilityModel::MarkovTransition` (`REQUIREMENTS.md` R6). The module header is
explicit: it *"intentionally does not perform neural-network inference."* This is
the associative re-expression of probabilistic reasoning — symbolic records, not
learned weights.

---

## 5. Symbolic machine learning

### 5.1 Rule synthesis / inductive logic programming — **applied**
ID3/version-spaces/ILP map to deriving rules from data: `src/rule_synthesis.rs`
synthesizes substitution rules, keeping the learned artifact a **readable rule**
(the interpretability the article credits to decision trees), not an opaque
model.

### 5.2 Case-based reasoning — **applied**
The article's CBR practice = reuse a past solution for a similar problem. The
solver's history lookup (`src/solver.rs` step 4) and the offline-first knowledge
oracle (`src/knowledge.rs` `CodingOracle`, `REQUIREMENTS.md` R289) reuse cached
prior answers as cases.

### 5.3 Natural-language skill compilation — **applied**
ACT-R "knowledge compilation" analog: `src/skill_compiler.rs` compiles
natural-language skills into deterministic trigger/response rule packages
(`ARCHITECTURE.md` §9, row 9).

---

## 6. Engineering best practices the article credits to symbolic AI

### 6.1 Explainability / glass box — **applied** (flagship)
The article's headline strength: *"explanations could be provided … by explaining
which rules were applied,"* and the glass-box transparency of inspectable
classifiers. Every answer is a **projection of the append-only event log**
(`src/event_log.rs`), each answer carries a `trace:` pointer
(`src/solver.rs` step 11), and the trace/graph is exposed via `/v1/graph`. "Why
did you answer that?" is answerable by construction — the strongest single
argument the issue makes for the symbolic-AI framing.

### 6.2 Separation of domain knowledge from procedural code — **applied**
The article calls out separating procedural code from domain knowledge for
reusability. `VISION.md` §"Data Is The Interface" (line 162) and the split
between `data/seed/*.lino` (knowledge) and `src/` (the engine) realize exactly
this. Adding knowledge is editing data, not code — the classical
knowledge-base/inference-engine separation.

### 6.3 Knowledge-acquisition bottleneck mitigation — **applied** (design value)
The article's most-cited weakness. The stack's answer is a **small seed +
on-demand growth**: public KBs are a bounded cache (1% or 512-item cap,
`REQUIREMENTS.md` R290; `src/knowledge.rs` `cache_capacity`), not a hand-built
Cyc. This is a deliberate design stance against the bottleneck, documented in the
case-study Risks table.

### 6.4 Sample efficiency & verifiability — **applied**
The article credits symbolic systems with needing little data and offering
formal guarantees. The stack needs **no training data** (it is rule- and
KB-driven) and is **deterministic** for a given config/impulse
(`src/solver.rs` header), so its behavior is testable to the byte — see the
1,440/1,440 ratchet (`REQUIREMENTS.md` R297) and cross-runtime parity
(`REQUIREMENTS.md` R291).

---

## 7. Neuro-symbolic integration — **partial, by design**

`formal-ai` is deliberately pure-symbolic, but the article's neuro-symbolic
framing (Kahneman System 1/2; Kautz's six categories) is still actionable. The
project makes one conscious borrowing in the *opposite* direction from typical
hybrids: it lifts the **softmax-temperature** idea from neural nets and applies
it to **discrete, Wikidata-anchored interpretations** (`VISION.md` §"Formalization
And Temperature", line 90; `ARCHITECTURE.md` §6 "Temperature-Based Interpretation
Selection", which uses a "softmax … content-hash-seeded draw"). Strength borrowed, substrate kept symbolic. The neural half is out of
current scope (`NON-GOALS.md`), and the 2024–2026 survey literature (research
file §9) is tracked for when/if that changes.

---

## 8. Summary

| # | Best practice | Status |
|---|---|---|
| 1.1 | Symbolic, human-readable representation | applied |
| 1.2 | Semantic networks | applied |
| 1.3 | Ontologies & grounding | applied |
| 1.4 | Production rules (if–then) | applied |
| 1.5 | Frames & scripts | partial |
| 2.1 | Forward / backward chaining | applied |
| 2.2 | Automated theorem proving | applied |
| 2.3 | Truth maintenance / non-monotonic | applied |
| 2.4 | Meta-level reasoning | partial |
| 3.1 | Means-ends / decomposition planning | applied |
| 3.2 | SAT / constraint solving | proposed |
| 4.1 | Bayesian / Markov (symbolic) | applied |
| 5.1 | Rule synthesis / ILP | applied |
| 5.2 | Case-based reasoning | applied |
| 5.3 | Skill compilation | applied |
| 6.1 | Explainability / glass box | applied |
| 6.2 | Knowledge ⟂ procedure separation | applied |
| 6.3 | Knowledge-acquisition bottleneck mitigation | applied |
| 6.4 | Sample efficiency & verifiability | applied |
| 7 | Neuro-symbolic integration | partial |

**15 applied, 4 partial, 1 proposed.** The applied rows are existing code now
made citable; the partials are honest scope boundaries; the single proposed row
has a named reuse target.

---

## 9. Proposed work (the gaps), with reuse targets

No best practice is left both unimplemented and unplanned (R303). The
non-"applied" rows, each with the *existing component* to reuse rather than
re-build:

1. **SAT / constraint solving (3.2, proposed).** Reuse a Rust SAT/CSP crate
   ([`splr`](https://crates.io/crates/splr),
   [`varisat`](https://crates.io/crates/varisat)) behind the
   `src/proof_engine/decision.rs` delegation seam — the same pattern that already
   wraps `link-calculator` for arithmetic. New associative package, no engine
   rewrite.
2. **Frames with default inheritance (1.5, partial).** Extend concept seeds with
   slot-default + override links; the doublet store already supports the edges,
   so this is data + a small resolver, not new infrastructure.
3. **Rule-driven meta-reasoning (2.4, partial).** Promote the solver's built-in
   strategy selection into `when … then …` meta-rules so strategy choice is as
   inspectable as object-level reasoning — reuses the existing substitution
   engine.
4. **RDF/OWL export bridge (1.2/1.3, future).** Because Links Notation is
   semantic-network-equivalent, a serializer to RDF triples would make the store
   interoperable with standard ontology tooling — reuses existing exporters.

These are recorded here (and in the case-study `README.md` §6) so the audit is a
working list, not a marketing page (`NON-GOALS.md`).

---

### Sources

The article and supporting references are listed with links in
[`raw-data/online-research.md`](raw-data/online-research.md). Primary source:
[Symbolic artificial intelligence — Wikipedia](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence).
