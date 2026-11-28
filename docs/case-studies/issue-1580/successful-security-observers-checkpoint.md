# Successful hosted security observers, 2026-11-21

Published head `27e895c1103926b4f73862c0eb15fa77fb505089` completes
[Security run 44860038851](https://github.com/link-assistant/formal-ai/actions/runs/44860038851).
All 142 downloaded producer payloads equal their retained compressed originals;
their bytes, newlines, raw/compressed hashes and source identity verify. The
22 retained command streams also verify. The two original SARIF reports have
zero findings, zero warning/error execution notifications and successful
invocations. Rust retains informational diagnostics, including all 631 original
unexpanded macro positions. Aggregate identity verification and structured
assessment do not claim literal reading of every retained line.

Both opt-in queries compile and evaluate with their unchanged four-minute
step bounds, two threads, 4 GiB policy and resolved 2,607 MiB loading heap.
Neither emits query runtime diagnostics. The macro-context query takes
26.1 seconds to compile and 59.3 seconds to evaluate. Its 122,574 rows span
2,232 source paths: 42,436 expanded outside another expansion, 79,507 expanded
inside one, and 631 unexpanded outside one. Every original SARIF position
matches the final category. This is positional correspondence, with no
individual child-to-expansion semantic or dataflow qualification.

The included-string query takes 22.3 seconds to compile and 3m30s to evaluate.
Its 2,179 rows span 258 source paths. Of these, 1,930 have literal expansions:
266 have CFG nodes and 1,664 do not. Another 248 are unexpanded without CFG
nodes, while one nonliteral expansion has a CFG node. Original complete
literal text remains in the retained table. A partitioned position inventory
records every row's literal byte count/hash, source coordinates and explicit
expansion/CFG states. Neither a payload string nor a CFG node establishes
complete per-child security semantics. No local CodeQL execution is claimed.

These observations qualify hosted execution after removing the duplicate
inverse option. They do not establish cache eviction as the earlier timeout's
cause. The observer's finite execution budget and strict security gate stay
unchanged. The first assessment helper used state names that differed from
the retained query's actual vocabulary; its assertion failed before writing
assessment output. The corrected retry uses the query's literal state names.
Both attempts are retained separately and earn zero independent reasoning
credit.

The maintained committable-path source scanner still reports 92 warnings and
zero ordinary/embedded-data violations. An initial scan without that maintained
selection included five ignored seed copies and reported 97 warnings; it is
retained as an observation of a different input set, not a product regression.
No scanner limits or exclusions change. Source warnings, complete template
practices, original/live acceptance and final PR review remain unfinished.
The PR stays draft and unmerged.

Evidence: `experiments/issue-1580/evidence/ci/codeql-producer-27e895c1/` and
`experiments/issue-1580/evidence/checks/current-codeql-observation-assessment/`.
