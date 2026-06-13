# Online research — Symbolic AI and the associative stack

> Summarized and cited per the repository documentation non-goal: *"Research
> notes should not copy large external texts; they should summarize and cite
> sources."* ([NON-GOALS.md](../../../../NON-GOALS.md))

Primary source requested by issue #451:
[**Symbolic artificial intelligence** — Wikipedia](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence).
Supporting sources are cited inline.

---

## 1. What "Symbolic AI" means

Symbolic AI (a.k.a. *classical AI*, *logic-based AI*, or, retrospectively,
**GOFAI** — "Good Old-Fashioned Artificial Intelligence", a term coined by John
Haugeland) is *"the collection of all methods in artificial intelligence
research that are based on high-level **symbolic (human-readable)
representations** of problems, logic, and search."*
([Wikipedia: Symbolic AI](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence))

It rests on the **physical symbol system hypothesis** (Allen Newell & Herbert
Simon, 1976): *a physical symbol system has the necessary and sufficient means
for general intelligent action.* Intelligence is modelled as the manipulation of
human-readable symbols by formal rules, rather than as numeric optimization of
hidden weights.

This is the exact niche `formal-ai` occupies: the README opens by calling it
*"a Rust implementation of a symbolic, deterministic assistant that exposes
OpenAI-shaped interfaces without neural-network inference."* The project is, by
the article's own definition, a symbolic-AI system.

## 2. History (the boom/winter cycles)

| Period | What happened | Source term |
|---|---|---|
| First AI summer (1956–1974) | Logic Theorist, GPS, Lisp, search, early NLP | "reasoning as search" |
| First AI winter (1974–1980) | Lighthill Report; **combinatorial explosion**; toy-problem critique | "combinatorial explosion" |
| Knowledge / expert-system boom (1980–1987) | DENDRAL, MYCIN, XCON; *"In the knowledge lies the power"* (Feigenbaum) | **Knowledge Principle** |
| Second AI winter (1987–1993) | Lisp-machine market collapse; expert systems costly to maintain | **knowledge-acquisition bottleneck** |
| Foundations (1993–2011) | Bayesian networks, HMMs, ILP, PAC learning add rigor | "uncertainty + rigor" |
| Deep-learning era (2011–present) | Connectionism dominates perception; **neuro-symbolic** integration emerges | "third wave" |

([Wikipedia: Symbolic AI](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence);
[Wikipedia: History of AI](https://en.wikipedia.org/wiki/History_of_artificial_intelligence))

## 3. The techniques the article enumerates

Grouped as the article groups them; each is a candidate "best practice" to map
onto the associative stack (done in
[`../symbolic-ai-best-practices.md`](../symbolic-ai-best-practices.md)).

- **Knowledge representation**: semantic networks, frames (Minsky), scripts
  (Schank), production rules (if–then), ontologies (WordNet, YAGO, DOLCE),
  description logic, OWL.
- **Reasoning / inference**: forward chaining (data→conclusions; CLIPS, OPS5,
  Jess, Drools), backward chaining (goal→data; Prolog), meta-level reasoning
  (Soar), blackboard architectures, truth-maintenance / non-monotonic reasoning.
- **Search & planning**: BFS/DFS, **A\*** (complete + optimal heuristic search),
  minimax / alpha-beta, Monte-Carlo tree search, SAT (DPLL, CDCL, WalkSAT),
  GPS / means-ends analysis, STRIPS, Graphplan, Satplan.
- **Uncertainty**: Bayesian networks (Pearl), hidden Markov models, Markov logic
  networks, probabilistic soft logic, fuzzy logic (Zadeh).
- **Symbolic machine learning**: version-space learning, decision trees
  (ID3/C4.5 — interpretable rules), inductive logic programming, case-based
  reasoning, genetic programming, ACT-R knowledge compilation.
- **Languages**: Lisp (REPL, GC, higher-order functions), Prolog (Horn clauses,
  unification, backtracking).

## 4. Strengths the article credits to symbolic AI

1. **Explainability** — *"explanations could be provided for an inference by
   explaining which rules were applied."*
2. **Verifiability / formal guarantees** — e.g. A\*'s completeness + optimality.
3. **Sample efficiency** — knowledge-intensive systems need far less data than
   neural ones.
4. **Transparency ("glass box")** — decision trees and deductive classifiers are
   inspectable, unlike "black box" networks.
5. **Abstraction** — Gary Marcus: *"Too much of useful knowledge is abstract to
   make do without tools that represent and manipulate abstraction."*
6. **Domain-expertise capture & reuse** — expert systems preserve and re-apply
   specialist knowledge (e.g. GUIDON reusing MYCIN's base).

Every one of these is a *stated design value* of `formal-ai` (VISION:
"transparent reasoning… every step traceable to links and source events";
NON-GOALS: "GPU-required neural inference is not a project target").

## 5. Weaknesses / criticisms the article names

- **Knowledge-acquisition bottleneck** — hand-encoding knowledge is slow and
  expensive; bases are hard to keep current.
- **Common-sense reasoning** — implicit knowledge is hard to formalize (Cyc
  spent decades on it).
- **The frame / qualification problem** (McCarthy & Hayes, 1969) — enumerating
  all preconditions/effects of actions is intractable.
- **Brittleness** — fails sharply off-domain; no graceful degradation.
- **Scaling / combinatorial explosion** — the Lighthill critique.

These are the *risks* the case study's solution plans must consciously mitigate
(small seeds + on-demand growth, public-knowledge cache, bounded decomposition
depth). See [`../README.md`](../README.md) §"Risks".

## 6. Controversies the article documents

- **Neats vs. scruffies** — formal-logic generality (McCarthy) vs. ad-hoc
  domain knowledge (Minsky/Schank). Cyc is the canonical "scruffy" system.
- **Symbolic vs. connectionist** — implementationism / radical / moderate
  connectionism. Marcus argues the animus against symbols is *"more sociological
  than philosophical."*
- The current consensus the article reports: **integration, not winner-take-all.**

## 7. Neuro-symbolic AI (the "third wave")

The article frames integration through **Daniel Kahneman's System 1 / System 2**
(*Thinking, Fast and Slow*): fast intuitive pattern-matching (neural) vs. slow
deliberate reasoning (symbolic) — *"both are needed."*

**Henry Kautz's taxonomy** of neuro-symbolic architectures
([Wikipedia: Neuro-symbolic AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI)):

1. `Symbolic → Neural → Symbolic` (standard NLP; BERT/GPT)
2. `Symbolic[Neural]` (AlphaGo: symbolic MCTS calling a neural evaluator)
3. `Neural | Symbolic` (neural perception feeds a symbolic reasoner)
4. `Neural : Symbolic → Neural` (symbolic system generates neural training data)
5. `Neural_{Symbolic}` (networks compiled from symbolic rules — Logic Tensor
   Networks, Neural Theorem Prover)
6. `Neural[Symbolic]` (a neural model calls a symbolic engine)

`formal-ai` is deliberately at the **pure-symbolic** end (no neural inference),
but the article's framing is still actionable: it consciously borrows the
*softmax-temperature* idea from neural nets and re-applies it to **discrete,
Wikidata-anchored interpretations** (VISION §"Formalization And Temperature") —
a symbolic re-implementation of a neural mechanism, which is exactly Kautz's
"borrow the strength, keep the substrate" spirit pointed the other way.

## 8. Semantic networks — the direct map to the "associative" stack

A **semantic network** is *"a knowledge base that represents semantic relations
between concepts in a network"* — vertices = concepts, labeled edges = relations
— and it is *"cognitively based,"* contributing **spreading activation,
inheritance, and nodes-as-proto-objects**.
([Wikipedia: Semantic network](https://en.wikipedia.org/wiki/Semantic_network))

This is the precise classical-AI name for what VISION.md calls *"an inspectable
**associative network of links**."* `formal-ai`'s doublet store **is** a
semantic network; Links Notation **is** its human-readable serialization
(cf. RDF triples / conceptual graphs, which the article lists as semantic
networks with *"expressive power equal to or exceeding standard first-order
predicate logic."*). WordNet, ConceptNet, RDF triples, and Google's Knowledge
Graph are the named prior art.

## 9. Additional / recent facts (2024–2026 domain literature)

The "third wave" framing and the LLM-grounding angle are corroborated by recent
surveys:

- *Towards Data- and Knowledge-Driven AI: A Survey on Neuro-Symbolic Computing*,
  IEEE TPAMI (2025) —
  <https://www.computer.org/csdl/journal/tp/2025/02/10721277/2179549p9QY>
- *Neuro-Symbolic AI: Explainability, Challenges, and Future Trends*, arXiv
  2411.04383 (2024) — <https://arxiv.org/html/2411.04383v1> — notes that the
  *format of the neural↔symbolic conversion* governs explainability (the more
  readable the intermediate representation, the more explainable the system).
  `formal-ai`'s intermediate representation is Links Notation text — maximally
  readable by construction.
- *Enhancing Large Language Models through Neuro-Symbolic Integration and
  Ontological Reasoning*, arXiv 2504.07640 (2025) —
  <https://arxiv.org/html/2504.07640v1> — translating model outputs into logical
  statements and checking them against **ontological axioms (OWL/RDF)** to detect
  and *explain* rule violations. Mirrors `formal-ai`'s
  formalize→verify→trace loop, but anchored on Wikidata instead of a bespoke
  ontology.
- *A Review of Neuro-Symbolic AI Integrating Reasoning and Learning…*,
  ScienceDirect S2667305325000675 (2025) —
  <https://www.sciencedirect.com/science/article/pii/S2667305325000675>.

Common thread across the 2025 literature: **LLM + knowledge graph** is named the
most promising route to *"trustworthy, explainable, and interoperable"* AI. The
associative-link store is `formal-ai`'s knowledge graph; the project's bet is
that the *graph + deterministic reasoning* half can stand on its own for the
task classes it targets, while keeping the door open to the neural half later.

---

### Source list

- Wikipedia, *Symbolic artificial intelligence* —
  <https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence>
- Wikipedia, *Neuro-symbolic AI* —
  <https://en.wikipedia.org/wiki/Neuro-symbolic_AI>
- Wikipedia, *Semantic network* —
  <https://en.wikipedia.org/wiki/Semantic_network>
- Wikipedia, *Physical symbol system* —
  <https://en.wikipedia.org/wiki/Physical_symbol_system>
- Wikipedia, *History of artificial intelligence* —
  <https://en.wikipedia.org/wiki/History_of_artificial_intelligence>
- IEEE TPAMI survey (2025) —
  <https://www.computer.org/csdl/journal/tp/2025/02/10721277/2179549p9QY>
- arXiv 2411.04383 (2024) — <https://arxiv.org/html/2411.04383v1>
- arXiv 2504.07640 (2025) — <https://arxiv.org/html/2504.07640v1>
- ScienceDirect S2667305325000675 (2025) —
  <https://www.sciencedirect.com/science/article/pii/S2667305325000675>
</content>
</invoke>
