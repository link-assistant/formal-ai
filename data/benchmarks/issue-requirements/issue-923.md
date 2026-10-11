**Problem.** Reasoning must stay free of neural networks while the
formal-reasoning implementation grows to cover all existing test cases
and much more (issue #914). The proof engine currently covers
propositional SAT and linear arithmetic; equality reasoning, rule-based
inference at scale, and richer decision procedures are missing, and no
external reasoning benchmark corpus is exercised.

**Approach.** Widen the symbolic kernel in verified steps: equality and
rewriting through e-graph saturation (egg and egglog are MIT-licensed
Rust libraries), scalable rule inference through embedded Datalog
(Ascent) or pure-Rust Prolog (Scryer) where a dependency is justified
against reimplementation, and SMT-style procedures either native or via
the `z3` bindings behind an optional feature. Every addition lands with
benchmark cases drawn from the corpora surveyed in the issue #914 online
research, scored honestly through the #698 external-benchmark harness,
and the existing suite is the floor.

**Existing components.** `src/proof_engine/` (SAT, linear, library,
presenters); `src/external_benchmarks/` (#698); `src/probability.rs` and
`src/world_model.rs`; egg, egglog, Ascent, Scryer Prolog, z3 and cvc5
Rust bindings (licenses recorded in the online research).

**Acceptance criteria.**
- At least two new reasoning capabilities (for example equality
  saturation and rule-based inference) land with external benchmark
  scores recorded in `data/benchmarks/`.
- No neural inference enters the dependency tree; any new dependency is
  license-checked and feature-gated.
- All pre-existing reasoning tests still pass unchanged.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

