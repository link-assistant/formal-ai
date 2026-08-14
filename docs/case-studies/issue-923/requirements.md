# Issue 923 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R923-1 | Add at least two symbolic capabilities beyond SAT and linear arithmetic. | `decision/equality.rs`, `decision/rules.rs`, `equality_saturation_proves_a_non_numeric_rewrite`, and `datalog_rule_inference_proves_transitive_reachability`. |
| R923-2 | Exercise external reasoning examples and record honest scores under `data/benchmarks/`. | Pinned egg and Ascent adapters, exact 20/20 and 5/5 result rows, ratchet floors, and `pinned_upstream_symbolic_suites_pass_end_to_end`. |
| R923-3 | Add no neural inference; license-check and feature-gate new dependencies. | `egg` 0.11.0 is optional, MIT-licensed, deterministic, and has default features disabled; Datalog is in-tree. |
| R923-4 | Preserve existing reasoning behavior and report limits honestly. | No pre-existing reasoning case is removed or relaxed; focused soundness tests require inconclusive search/limit behavior, reject unsafe Datalog rules, and reject equality-to-linear fallthrough. |
| R923-5 | Preserve issue, PR, source, score, and self-hosting traceability. | Raw GitHub JSON, online research, root docs, changelog, exact Agent CLI leaf, and its replay script are checked by `docs_requirements_issue_923`. |

## Reviewed Leaf Accounting

The five reviewed leaves are: (1) equality engine, (2) Datalog engine, (3)
pinned adapters and score ledger, (4) tests, CI, and documentation, and (5) the
symbolic-kernel invariant. Real Formal AI/Agent CLI session
`ses_001f733ceffe5UboLW4JATfkoZ` authored leaf 5. The other four leaves are
reported as manual work, giving an honest one-of-five self-authorship share.
