## Issue #919 Research-Driven Coding Procedures

Issue [#919](https://github.com/link-assistant/formal-ai/issues/919) closes the
procedure-learning gap between named program synthesis failures and the
existing cached research and bounded workspace execution machinery. Detailed
design, alternatives, standards research, and verification live in
`docs/case-studies/issue-919/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R919-1 | A real coding synthesis miss must enter the loop as its recorded stable skill-gap identity. | `tests/unit/issue_919.rs` starts with the real unsupported Ruby `count_to_three` request, asserts `write_program_skill_gap`, and carries the matching `program_skill_gap` identity into `CodingResearchGap`. |
| R919-2 | Research must be query-planned, provenance-bearing, cacheable, and formalized into the meta-language before compilation. | `coding_research_learning::research_coding_skill_gap` derives the query, uses `execute_source_research`, requires the licensed v1 procedure source shape, and emits content-addressed Links Notation. |
| R919-3 | A researched procedure must be marked and pass the same bounded execution verification as a hand-seeded procedure before it is kept. | `origin research` candidates use #897's `execute_workspace_rewrite`; exact expected output and named review are gates, and only `execution_verified` procedures enter the ledger. |
| R919-4 | Full provenance and deterministic offline replay must survive CI. | The ledger retains query, URL, declared SPDX license, fetch time, source hash, formalization, executor, output hash, step count, and reviewer. The regression proves a default-offline client makes no transport calls and reproduces the proposal, id, ledger, and output from cache. |
| R919-5 | Failed rounds must remain non-executable and update the gap to drive the next research round. | A mismatched execution rejects the candidate, leaves the ledger empty, appends query/reason to the gap, and schedules `alternative evidence round 2`. |
| R919-6 | The loop must follow a data-authored contract and build on the completed E69 dependency. | `data/meta/coding-research-learning-contract.lino` pins the source, provenance, execution, live/offline, review, and recovery boundaries; E69 issue #916 / PR #966 is merged. |
