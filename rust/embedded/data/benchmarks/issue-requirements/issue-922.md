**Problem.** The system must be able to learn the universal
problem-solving algorithm (issue #914) — not only execute it. Today the
11-step loop is data (`data/meta/recursive-core-recipe.lino`) run by
`src/recipe_interpreter.rs`, and `src/algorithm_discovery.rs` can
discover parameterized algorithms from traces, but nothing feeds solved
and failed problem experience back into the recipe or the method
registry.

**Approach.** Mine the append-only event log for recurring step
sequences across solved problems, compress them into candidate method
abstractions (corpus-guided abstraction in the DreamCoder line; the Rust
`stitch_core` crate is the external reference for making the compression
fast), and register candidates in the method registry strictly as
proposals. Adoption goes only through the #656 benchmark-gated promotion
protocol with human confirmation, so self-modification stays gated while
the core loop becomes improvable from experience.

**Existing components.** `src/recipe_interpreter.rs`;
`src/method_registry.rs` and `src/meta_method_dispatch.rs`;
`src/algorithm_discovery.rs` (inert, green-gate approved);
`src/learning_cycle.rs` and adoption ledgers; `src/promotion.rs` (#656);
Stitch as external reference.

**Acceptance criteria.**
- At least one method abstraction is proposed from real event-log
  traces, survives the benchmark gate, and is adopted into the registry
  through the human-confirmed promotion path.
- Proposals are inert until promoted; rejected proposals remain recorded
  with reasons.
- The recipe-equals-source test
  (`tests/unit/specification/recursive_core_recipe.rs`) still passes
  after adoption.
- The regression floor holds.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

