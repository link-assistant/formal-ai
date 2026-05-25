# Online Research — Issue #244 Vision Planning

This note collects external facts and prior art relevant to issue #244's vision:
a deterministic, link-native problem solver that learns a *universal problem
solving algorithm* and translates between natural and formal languages **without
using neural networks for the reasoning itself**. Citations are summarized, not
copied, per `NON-GOALS.md` ("research notes should not copy large external
texts; they should summarize and cite sources").

## 1. Neuro-symbolic and symbolic reasoning over knowledge graphs

Most 2024–2025 work on "reasoning over knowledge graphs" is *neuro-symbolic*:
it pairs a neural learner with a symbolic component to get robustness plus
interpretability. formal-ai deliberately takes the **symbolic-only** branch of
that spectrum — the knowledge graph (doublet links) *is* the model, and
inference is graph rewriting and rule substitution rather than learned weights.
The surveys below are useful to position the project and to borrow evaluation
framing (logic/reasoning ~35%, knowledge representation ~44% of studies), while
explicitly rejecting the neural learner.

- Neurosymbolic AI for Reasoning Over Knowledge Graphs: A Survey — <https://pubmed.ncbi.nlm.nih.gov/39024082/>
- Towards Unified Neurosymbolic Reasoning on Knowledge Graphs (2025) — <https://arxiv.org/abs/2507.03697>
- A review of neuro-symbolic AI integrating reasoning and learning — <https://www.sciencedirect.com/science/article/pii/S2667305325000675>

Takeaway: the literature treats symbolic KG reasoning as the *interpretable*
half of intelligence. Our differentiator is doing the whole loop symbolically
and keeping every step inspectable as Links Notation.

## 2. Abstract Wikipedia / Wikifunctions — translation through language-independent meaning

This is the closest public analogue to VISION.md's "translate through
link-native meanings" idea. Abstract Wikipedia stores **language-independent
abstract content** anchored on Wikidata items/lexemes; Wikifunctions hosts
**renderers** — functions that turn that abstract content into natural-language
sentences using the linguistic data in Wikidata. This validates our pipeline
`formalize → meaning → deformalize`: a meaning anchored on a Wikidata Q/P id can
be rendered into any language whose labels/lexemes exist.

- Building a Multilingual Wikipedia (CACM, Vrandečić) — <https://cacm.acm.org/opinion/building-a-multilingual-wikipedia/>
- Abstract Wikipedia (Meta-Wiki) — <https://meta.wikimedia.org/wiki/Abstract_Wikipedia>
- Abstract Wikipedia / NLG system architecture proposal — <https://meta.wikimedia.org/wiki/Abstract_Wikipedia/Natural_language_generation_system_architecture_proposal>
- Wikifunctions FAQ — <https://www.wikifunctions.org/wiki/Wikifunctions:FAQ>

Takeaway: we should keep treating Wikidata P/Q ids as the meaning anchor and
Wiktionary/Wikipedia as per-language surfaces, and watch Wikifunctions
renderers as a possible source of deterministic per-language generation rules.

## 3. OpenCog AtomSpace / Hyperon — associative hypergraph memory + self-modifying rules

OpenCog's AtomSpace is a (hyper)graph database whose query engine is a graph
rewriting system and whose rule engine is a generalized rule-driven inferencing
system. OpenCog Hyperon adds the **Distributed AtomSpace** plus **MeTTa**, a
language for introspective, self-modifying programs over that graph. This is the
single closest prior art to formal-ai's "the associative network is the AI" plus
the five rule shapes in VISION.md (pure-data rules → compiled handlers →
dynamically compiled code → natural-language skills).

- AtomSpace (graph DB + graph rewriting) — <https://github.com/opencog/atomspace>
- AtomSpace wiki — <https://wiki.opencog.org/w/AtomSpace>
- OpenCog Hyperon (DAS + MeTTa) — <https://hyperon.opencog.org/>

Takeaway: our doublet store + trigger/substitution computation model is a
restricted, reviewable cousin of AtomSpace. We use **doublets** (Link Foundation)
instead of triplets/hypergraph, and Links Notation as the human-reviewable text
layer. AtomSpace/MeTTa is the reference for rule-as-data and self-modification.

Cognitive-architecture context (Soar, ACT-R, Sigma, MicroPsi) is the lineage for
the "universal problem-solving loop" — symbolic production systems that run the
same recognize→decide→act cycle for every goal, which is exactly the
domain-agnostic loop VISION.md asks for. Newell & Simon's General Problem Solver
is the historical root of "one algorithm, any task via means-ends analysis".

## 4. Program synthesis & automated theorem proving — deterministic verification

For the "generate code + tests, then verify" and "formal reasoning that covers
the test cases" parts of the vision, the mature deterministic tools are SMT
solvers and interactive/automated theorem provers:

- Z3 (SMT solver, MIT-licensed) — used widely for symbolic analysis/verification.
- Lean (interactive theorem prover, Calculus of Inductive Constructions) — <https://galois.com/blog/2018/07/the-lean-theorem-prover-past-present-and-future/>
- Canonical: type-inhabitation/program synthesis in Lean (2025) — <https://arxiv.org/abs/2504.06239>
- Program Synthesis in Saturation (first-order prover synthesis) — <https://arxiv.org/pdf/2402.18962>

Takeaway: the current `src/proof_engine/` is a small classical-theorem registry.
The long-term "formal reasoning that covers all test cases and much more" goal
points toward integrating a real decision procedure (the repo already references
`link-assistant/relative-meta-logic`) and/or an SMT backend for arithmetic and
constraint checks, rather than expanding a hand-written theorem table.

## 5. Repository-internal references (the building blocks we already cite)

- `link-foundation/doublets-rs` — long-term doublet store backend (planned, not yet a dependency).
- `link-foundation/doublets-web` — browser-side IndexedDB mirror (planned).
- `link-assistant/calculator` (`link-calculator` crate) — already integrated for arithmetic.
- `link-assistant/relative-meta-logic` — future formal-reasoning integration point.
- `link-foundation/lino-i18n`, `lino-objects-codec`, `lino-arguments` — Links Notation tooling already in use.

## Summary of how the prior art shapes the plan

| Vision element | Closest prior art | What we adopt / reject |
| --- | --- | --- |
| "Associative network is the AI" | OpenCog AtomSpace / Hyperon | Adopt graph-rewriting + rule-as-data; use doublets + Links Notation; reject neural learner. |
| Translation via language-independent meaning | Abstract Wikipedia + Wikifunctions | Adopt Wikidata P/Q anchors + per-language renderers; keep Wiktionary/Wikipedia surfaces. |
| Universal problem-solving loop | GPS, Soar, ACT-R | Adopt one domain-agnostic recognize→decide→act loop; keep it deterministic + traceable. |
| Formal reasoning over test cases | Lean / Z3 / saturation synthesis | Integrate a real decision procedure (relative-meta-logic / SMT) instead of a fixed theorem table. |
| Reasoning without neural nets | Symbolic-only branch of NeSy surveys | Keep the whole loop symbolic and inspectable; use the web (Wikidata/Wikipedia/Wiktionary) as a cache, not a teacher. |
