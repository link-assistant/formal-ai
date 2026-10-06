# Architecture: References

Part of the [architecture overview](../../ARCHITECTURE.md) (§17). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 17. References

- `VISION.md` — values, product story, north-star user experience.
- `GOALS.md` — what counts as success per surface.
- `NON-GOALS.md` — what we explicitly do not build.
- `REQUIREMENTS.md` — issue-by-issue implementation matrix (R1 … R558, plus per-issue blocks such as R499-1…R499-8, R914-1…R914-15, R1021-1…R1021-32, R1085-1…R1085-17, R1137-1…R1137-3, R710-01…R710-32, R710-D1…R710-D17, and R1138-B1…R1138-B12).
- `ROADMAP.md` — implementation-progress tracker mapping each `VISION.md` pillar to its real code status, closed planning batches, and remaining follow-up gaps.
- [`link-foundation/link-cli`](https://github.com/link-foundation/link-cli) — default native transactional storage library.
- [`linksplatform/doublets-rs`](https://github.com/linksplatform/doublets-rs) — physical doublet store embedded by link-cli.
- [`linksplatform/doublets-web`](https://github.com/linksplatform/doublets-web) — browser-side mirror.
- [`link-assistant/calculator`](https://github.com/link-assistant/calculator) — delegated calculator engine (`link-calculator` crate).
- [`link-foundation/relative-meta-logic`](https://github.com/link-foundation/relative-meta-logic) — future formal-reasoning integration.
- Wikidata (`https://www.wikidata.org/`) — public source of P/Q-ID anchors.
- Wikipedia (`https://*.wikipedia.org/`) — public source of per-language
  concept articles.
- Wiktionary (`https://*.wiktionary.org/`) — public source of per-language
  word and idiom entries.

### Domain background (symbolic AI)

- [Symbolic artificial intelligence](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
  — the field this project belongs to (GOFAI); the source of the best-practice
  audit in `docs/case-studies/issue-451/symbolic-ai-best-practices.md`.
- [Semantic network](https://en.wikipedia.org/wiki/Semantic_network) — the
  classical-AI name for the associative link store.
- [Physical symbol system](https://en.wikipedia.org/wiki/Physical_symbol_system)
  — Newell & Simon's hypothesis underlying the link-as-symbol model.
- [Neuro-symbolic AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI) — the
  integration framing the project tracks while staying pure-symbolic.
- [Boolean satisfiability problem](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
  and the [DPLL algorithm](https://en.wikipedia.org/wiki/DPLL_algorithm) — the
  decision procedure the propositional engine delegates to for claims wider than
  the truth-table limit. The in-house, dependency-free DPLL search lives in
  `rust/src/proof_engine/decision/sat.rs`; wide claims are
  [Tseitin-encoded](https://en.wikipedia.org/wiki/Tseytin_transformation) to CNF
  in `rust/src/proof_engine/decision/boolean.rs` before being handed to it.
- Issue #923 widens the same decision boundary with bounded equality saturation
  in `rust/src/proof_engine/decision/equality.rs` and a bounded, function-free
  Datalog least-fixed-point evaluator in `rust/src/proof_engine/decision/rules.rs`.
  Equality uses the optional MIT-licensed `egg` dependency behind the default
  `equality-saturation` feature; an exhausted e-graph search remains
  inconclusive rather than being reported as a disproof. Rule programs carry
  explicit `facts`, `rules`, and a ground `query`; resource ceilings turn into
  an inconclusive result instead of an incomplete-model counterexample.

### Symbolic world models and contexts (issue #649)

- The design case study in `docs/case-studies/issue-649/README.md` audits how the
  associative stack realizes symbolic **world models**: a **current-state** and a
  **target-state** context, their difference, context **merge/split**, and
  **predicting the consequences of an action** — each context being a **links
  network** rather than an embedding. It maps the request onto the classical
  prior art ([STRIPS/PDDL](https://en.wikipedia.org/wiki/Stanford_Research_Institute_Problem_Solver)
  planning, [truth-maintenance systems](https://en.wikipedia.org/wiki/Reason_maintenance)
  JTMS/ATMS, and [AGM belief revision](https://en.wikipedia.org/wiki/Belief_revision))
  and onto [relative-meta-logic](https://github.com/link-foundation/relative-meta-logic),
  whose kernel already lives in `rust/src/relative_meta_logic.rs`. Statement
  dependency edges and the change-driven recalculation cascade reuse
  `SubstitutionGraph::apply_rules`; the concept-by-concept status is in
  `docs/case-studies/issue-649/world-model-mapping.md`.

### Usage-weighted associative persistence (issue #686)

- `rust/src/associative_persistence.rs` keeps a **persistent** version of
  **meta-language expressions** saved in an **associative links network**: an
  `AssociativeMemory` stores each expression as a content-addressed node (via
  `stable_id`, so one meaning is one node) in an embedded `SubstitutionGraph`,
  counts **usages (reads)** and **changes (writes)** per expression, and derives an
  independent usage signal from each node's **incoming and outgoing link degree**.
  A single `retention_score` (reads + writes + in-degree + out-degree, under
  configurable `RetentionWeights`) drives an LFU-style policy so the **most used,
  most changed, and most connected** knowledge **persists longest**; eviction
  forgets the lowest-scored first. Everything — expressions, read/write counts, and
  associations — is **a link** (never a separate edge/vertex type) and serializes to
  Links Notation. The design case study in
  `docs/case-studies/issue-686/README.md` maps the request onto its prior art
  ([Wikontic](https://huggingface.co/papers/2512.00590) entity-degree↔retrieval,
  [LFU cache replacement](https://en.wikipedia.org/wiki/Cache_replacement_policies),
  [reference counting](https://en.wikipedia.org/wiki/Reference_counting), and
  [degree centrality](https://en.wikipedia.org/wiki/Centrality#Degree_centrality)),
  and the concept-by-concept status is in
  `docs/case-studies/issue-686/persistence-mapping.md`. It generalizes the
  read-count LFU precursor already present in `rust/src/dreaming.rs` (`usage_counts`) and
  bridges to the issue #649 world model via `AssociativeMemory::from_context`.
