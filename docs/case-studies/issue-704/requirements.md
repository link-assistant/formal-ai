# Issue 704 requirements matrix

| Requirement | Implementation | Verification |
| --- | --- | --- |
| k candidates and deterministic seeds | `SolverConfig::draft_count`, `seed_for_draft` | three-draft and seed tests |
| Test every candidate | `evaluate_draft`, `draft:result` | event-count and pass/fail assertions |
| Least-action selection | `rank_passing_drafts` | comparison winner/cost assertions |
| Hierarchical backtracking | `select_composable_draft` | composition rejection unit test |
| Concurrent evaluation, ordered merge | `ordered_parallel_map`, index sort | timing and byte-determinism tests |
| Durable loser learning | `draft_failure`, max attempts 3 | trace assertions |
| Explain the choice | embedded comparison artifact and meta handler | four-language follow-up test |
| Industry ratchet | reach-26 benchmark case | minimum pass count 13 |
| Default compatibility | single-draft branch | no portfolio-event regression test |
