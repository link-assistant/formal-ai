# Issue 704 requirements matrix

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Domain-independent portfolio | `PortfolioLeaf` in `src/draft_portfolio.rs` | scripted-leaf engine tests |
| Strategies are data, not Rust | `data/seed/draft-strategies.lino`, `seed::draft_strategies` | catalog-order override test |
| Every strategy is a real generator | five generators in `src/solver_search/portfolio.rs` | strategy-disagreement assertions |
| More than one leaf uses the engine | `src/rule_synthesis_portfolio.rs` | second-leaf end-to-end test |
| k candidates and deterministic seeds | `SolverConfig::draft_count`, `seed_for_draft` | three-draft and seed tests |
| Test every candidate | `evaluate_draft`, `draft:result` | event-count and pass/fail assertions |
| Least-action selection | `rank_passing_drafts` | comparison winner/cost assertions |
| Hierarchical backtracking | `select_composable_draft` | composition rejection unit test |
| Concurrent evaluation, ordered merge | `ordered_parallel_map`, index sort | timing and byte-determinism tests |
| Durable loser learning | `draft_failure`, max attempts 3 | trace and retry-budget assertions |
| Failures are mined, not just logged | `src/dreaming/draft_failures.rs`, `DreamingPlan::draft_failures` | event log to dreaming-plan test |
| Explain the choice | embedded comparison artifact and meta handler | four-language follow-up test |
| Industry ratchet | reach-26 benchmark case | minimum pass count 13 |
| Default compatibility | single-draft branch | no portfolio-event regression test |
